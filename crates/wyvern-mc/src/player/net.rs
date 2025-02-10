use std::{
    collections::VecDeque,
    fmt::Debug,
    io::ErrorKind,
    net::IpAddr,
    time::{Duration, Instant},
};

use tokio::{io::AsyncWriteExt, net::TcpStream, sync::*};
use voxidian_protocol::packet::{
    DecodeError, PrefixedPacketDecode, Stage,
    c2s::handshake::C2SHandshakePackets,
    processing::{CompressionMode, PacketProcessing, SecretCipher},
    s2c::play::KeepAliveS2CPlayPacket,
};
use wyvern_actors::Actor;

use crate::{player::PlayerMessage, server::Server};

use super::{ConnectionData, ConnectionWithSignal, Player, data::PlayerData};

pub struct ConnectionStoppedSignal;

impl ConnectionData {
    pub fn connection_channel(
        stream: TcpStream,
        addr: IpAddr,
        server: Server,
    ) -> ConnectionWithSignal {
        let (signal_tx, signal_rx) = mpsc::channel(1);
        let (data_tx, data_rx) = mpsc::channel(256);

        tokio::spawn(ConnectionData::execute_connection(
            stream,
            addr,
            data_tx.clone(),
            data_rx,
            signal_tx,
            server,
        ));

        ConnectionWithSignal {
            player: Player { sender: data_tx },
            _signal: signal_rx,
        }
    }

    pub async fn execute_connection(
        stream: TcpStream,
        addr: IpAddr,
        sender: mpsc::Sender<PlayerMessage>,
        receiver: mpsc::Receiver<PlayerMessage>,
        signal: mpsc::Sender<ConnectionStoppedSignal>,
        server: Server,
    ) {
        let conn = ConnectionData {
            stream,
            addr,
            received_bytes: VecDeque::new(),
            bytes_to_send: Vec::new(),
            packet_processing: PacketProcessing {
                secret_cipher: SecretCipher::no_cipher(),
                compression: CompressionMode::None,
            },
            receiver,
            sender: sender.clone(),
            signal,
            stage: Stage::Handshake,
            connected_server: server,
            associated_data: PlayerData::default(),
        };

        conn.event_loop().await;
    }

    pub async fn event_loop(mut self) {
        loop {
            tokio::task::yield_now().await;
            let result = self.handle_incoming_bytes().await;
            if result.is_err() {
                log::info!("A player has disconnected. Stopping their connection data...");
                let _ = self.signal.send(ConnectionStoppedSignal).await;
                break;
            }
            self.handle_messages().await;
            self.read_incoming_packets().await;
            self.write_outgoing_packets().await;
            self.handle_messages().await;

            let now = Instant::now();
            if now > self.associated_data.last_sent_keep_alive + Duration::from_secs(5) {
                self.write_packet(KeepAliveS2CPlayPacket(10)).await;
                self.associated_data.last_sent_keep_alive = Instant::now();
            }
        }
    }

    pub async fn handle_incoming_bytes(&mut self) -> Result<(), ()> {
        let mut buf = [0; 512];
        let bytes_read = self.stream.try_read(&mut buf);
        match bytes_read {
            Ok(bytes_read) => {
                if bytes_read == 0 {
                    return Err(());
                }
                for byte in &buf[0..bytes_read] {
                    let byte = self
                        .packet_processing
                        .secret_cipher
                        .decrypt_u8(*byte)
                        .unwrap();
                    self.received_bytes.push_back(byte);
                }

                Ok(())
            }
            Err(e) if e.kind() == ErrorKind::WouldBlock => Ok(()),
            Err(_e) => Err(()),
        }
    }

    pub async fn read_incoming_packets(&mut self) {
        match self.stage {
            Stage::Handshake => {
                self.read_packets(async |packet: C2SHandshakePackets, this: &mut Self| {
                    let C2SHandshakePackets::Intention(packet) = packet;
                    this.stage = packet.intended_stage.into_stage();
                })
                .await;
            }
            Stage::Status => {
                self.status_stage().await;
            }
            Stage::Login => {
                self.login_stage().await;
            }
            Stage::Config => {
                self.configuration_stage().await;
            }
            Stage::Play => {
                self.play_phase().await;
            }
            Stage::Transfer => todo!("doesn't exist, this needs to be removed D:"),
        }
    }

    pub async fn write_outgoing_packets(&mut self) {
        loop {
            if self.bytes_to_send.is_empty() {
                break;
            }

            self.stream.write_all(&self.bytes_to_send).await.unwrap();

            self.bytes_to_send.clear();
            tokio::task::yield_now().await;
        }
    }

    pub async fn read_packets<T: PrefixedPacketDecode + Debug, F: AsyncFnOnce(T, &mut Self)>(
        &mut self,
        f: F,
    ) {
        match self
            .packet_processing
            .decode_from_raw_queue(self.received_bytes.iter().copied())
        {
            Ok((mut buf, consumed)) => {
                if consumed == 0 {
                    return;
                }

                let byte_cache = self.received_bytes.clone();
                let mut rv = Vec::with_capacity(consumed);
                for _ in 0..consumed {
                    rv.push(self.received_bytes.pop_front().unwrap());
                }

                let buf_copy = buf.clone();
                match T::decode_prefixed(&mut buf) {
                    Ok(packet) => {
                        f(packet, self).await;
                    }
                    Err(DecodeError::EndOfBuffer) => {
                        log::error!(
                            "The server has encountered a packet decoding error!

                        Buffer received: {:?}
                        Buffer after receiving: {:?}
                        Received bytes before consuming: {:?}
                        Bytes left to receive: {:?}
                        Bytes consumed: {:?}
                        ",
                            buf_copy.iter().collect::<Vec<u8>>(),
                            buf.iter().collect::<Vec<u8>>(),
                            byte_cache,
                            self.received_bytes,
                            rv
                        );
                    }
                    Err(e) => {
                        panic!("{:?}", e);
                    }
                }
            }
            Err(DecodeError::EndOfBuffer) => {}
            Err(e) => {
                panic!("err: {:?}", e);
            }
        }
    }
}
