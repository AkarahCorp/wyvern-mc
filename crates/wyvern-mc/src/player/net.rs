use std::{
    collections::VecDeque,
    fmt::Debug,
    io::ErrorKind,
    net::IpAddr,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use crate::{
    actors::{Actor, ActorResult},
    runtime::Runtime,
};
use async_net::TcpStream;
use flume::{Receiver, Sender};
use futures_lite::{AsyncReadExt, AsyncWriteExt};
use futures_util::future::Either;
use voxidian_protocol::packet::{
    DecodeError, PrefixedPacketDecode, Stage, c2s::handshake::C2SHandshakePackets,
    processing::PacketProcessing, s2c::play::KeepAliveS2CPlayPacket,
};

use crate::{player::PlayerMessage, server::Server};

use super::{ConnectionData, ConnectionWithSignal, Player, data::PlayerData};

pub struct ConnectionStoppedSignal;

impl ConnectionData {
    pub fn connection_channel(
        stream: TcpStream,
        addr: IpAddr,
        server: Server,
        stage: Arc<Mutex<Stage>>,
    ) -> ConnectionWithSignal {
        let (signal_tx, signal_rx) = flume::bounded(1);
        let (data_tx, data_rx) = flume::unbounded();

        Runtime::spawn(ConnectionData::execute_connection(
            stream,
            addr,
            data_tx.clone(),
            data_rx,
            signal_tx,
            server,
            stage.clone(),
        ));

        ConnectionWithSignal {
            player: Player { sender: data_tx },
            _signal: signal_rx,
            stage,
        }
    }

    pub async fn execute_connection(
        stream: TcpStream,
        addr: IpAddr,
        sender: Sender<PlayerMessage>,
        receiver: Receiver<PlayerMessage>,
        signal: Sender<ConnectionStoppedSignal>,
        server: Server,
        stage: Arc<Mutex<Stage>>,
    ) {
        let conn = ConnectionData {
            stream,
            addr,
            received_bytes: VecDeque::new(),
            bytes_to_send: Vec::new(),
            packet_processing: PacketProcessing::NONE,
            receiver,
            sender: sender.downgrade(),
            signal,
            stage,
            connected_server: server,
            associated_data: PlayerData::default(),
            private_key: None,
            verify_token: Vec::new(),
            public_key: None,
            props: Vec::new(),
        };

        conn.event_loop().await;
    }

    pub async fn event_loop(mut self) {
        loop {
            futures_lite::future::yield_now().await;
            let result = self.handle_incoming_bytes().await;
            if result.is_err() {
                log::info!("A player has disconnected. Stopping their connection data...");

                if let Some(dim) = self.associated_data.dimension {
                    let _ = dim.remove_entity(self.associated_data.uuid).await;
                }
                let _ = self.signal.send_async(ConnectionStoppedSignal).await;
                break;
            }
            self.handle_messages().await;
            let _ = self.read_incoming_packets().await;
            self.write_outgoing_packets().await;

            let now = Instant::now();
            if now > self.associated_data.last_sent_keep_alive + Duration::from_secs(5)
                && *self.stage.lock().unwrap() == Stage::Play
            {
                self.write_packet(KeepAliveS2CPlayPacket(10)).await;
                self.associated_data.last_sent_keep_alive = Instant::now();
            }
        }
    }

    pub async fn handle_incoming_bytes(&mut self) -> Result<(), ()> {
        let mut buf = [0; 512];

        match futures_util::future::select(
            async_io::Timer::after(Duration::from_millis(5)),
            self.stream.read(&mut buf),
        )
        .await
        {
            Either::Left(_) => Ok(()),
            Either::Right(bytes_read) => match bytes_read.0 {
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
            },
        }
    }

    pub async fn read_incoming_packets(&mut self) -> ActorResult<()> {
        let stage = *self.stage.lock().unwrap();
        match stage {
            Stage::Handshake => {
                self.read_packets(async |packet: C2SHandshakePackets, this: &mut Self| {
                    let C2SHandshakePackets::Intention(packet) = packet;
                    *this.stage.lock().unwrap() = packet.intended_stage.into_stage();
                    Ok(())
                })
                .await?;
            }
            Stage::Status => {
                self.status_stage().await?;
            }
            Stage::Login => {
                self.login_stage().await?;
            }
            Stage::Config => {
                self.configuration_stage().await?;
            }
            Stage::Play => {
                self.play_phase().await?;
            }
            Stage::Transfer => todo!("doesn't exist, this needs to be removed D:"),
        }
        Ok(())
    }

    pub async fn write_outgoing_packets(&mut self) {
        loop {
            if self.bytes_to_send.is_empty() {
                break;
            }

            self.stream.write_all(&self.bytes_to_send).await.unwrap();
            self.bytes_to_send.clear();
        }
    }

    pub async fn read_packets<
        T: PrefixedPacketDecode + Debug,
        F: AsyncFnOnce(T, &mut Self) -> ActorResult<()>,
    >(
        &mut self,
        f: F,
    ) -> ActorResult<()> {
        match self
            .packet_processing
            .decode_from_raw_queue(self.received_bytes.iter().copied())
        {
            Ok((mut buf, consumed)) => {
                if consumed == 0 {
                    return Ok(());
                }

                let byte_cache = self.received_bytes.clone();
                let mut rv = Vec::with_capacity(consumed);
                for _ in 0..consumed {
                    rv.push(self.received_bytes.pop_front().unwrap());
                }

                let buf_copy = buf.clone();
                match T::decode_prefixed(&mut buf) {
                    Ok(packet) => {
                        f(packet, self).await?;
                        Ok(())
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
                        Ok(())
                    }
                    Err(e) => {
                        panic!("{:?}", e);
                    }
                }
            }
            Err(DecodeError::EndOfBuffer) => Ok(()),
            Err(e) => {
                panic!("err: {:?}", e);
            }
        }
    }
}
