use std::{collections::VecDeque, fmt::Debug, io::ErrorKind, net::IpAddr};

use tokio::{io::AsyncWriteExt, net::TcpStream, sync::*};
use voxidian_protocol::packet::{
    DecodeError, PrefixedPacketDecode, Stage,
    c2s::handshake::C2SHandshakePackets,
    processing::{CompressionMode, PacketProcessing, SecretCipher},
};

use crate::{
    server::server::Server,
    systems::{
        events::ReceivePacketEvent,
        parameters::{Event, Param},
        typemap::TypeMap,
    },
};

use super::{data::PlayerData, message::ConnectionMessage};

pub struct ConnectionData {
    pub(crate) stream: TcpStream,
    #[allow(dead_code)]
    pub(crate) addr: IpAddr,
    pub(crate) received_bytes: VecDeque<u8>,
    pub(crate) bytes_to_send: Vec<u8>,
    pub(crate) packet_processing: PacketProcessing,
    pub(crate) sender: mpsc::Sender<ConnectionMessage>,
    pub(crate) receiver: mpsc::Receiver<ConnectionMessage>,
    pub(crate) signal: mpsc::Sender<ConnectionStoppedSignal>,
    pub(crate) stage: Stage,
    pub(crate) connected_server: Server,
    pub(crate) associated_data: PlayerData,
}

pub struct ConnectionStoppedSignal;

impl ConnectionData {
    pub fn connection_channel(
        stream: TcpStream,
        addr: IpAddr,
        server: Server,
    ) -> (
        mpsc::Sender<ConnectionMessage>,
        mpsc::Receiver<ConnectionStoppedSignal>,
    ) {
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

        (data_tx, signal_rx)
    }

    pub async fn execute_connection(
        stream: TcpStream,
        addr: IpAddr,
        sender: mpsc::Sender<ConnectionMessage>,
        receiver: mpsc::Receiver<ConnectionMessage>,
        signal: mpsc::Sender<ConnectionStoppedSignal>,
        server: Server,
    ) {
        let mut conn = ConnectionData {
            stream,
            addr,
            received_bytes: VecDeque::new(),
            bytes_to_send: Vec::new(),
            packet_processing: PacketProcessing {
                secret_cipher: SecretCipher::no_cipher(),
                compression: CompressionMode::None,
            },
            sender,
            receiver,
            signal,
            stage: Stage::Handshake,
            connected_server: server,
            associated_data: PlayerData::default(),
        };

        conn.event_loop().await;
    }

    pub async fn event_loop(&mut self) {
        loop {
            let result = self.handle_incoming_bytes().await;
            if let Err(_) = result {
                let _ = self.signal.send(ConnectionStoppedSignal).await;
                break;
            }
            self.read_incoming_packets().await;
            self.write_outgoing_packets().await;
            self.handle_messages().await;
            tokio::task::yield_now().await;
        }
    }

    pub async fn handle_incoming_bytes(&mut self) -> Result<(), ()> {
        let mut buf = [0; 256];
        let bytes_read = self.stream.try_read(&mut buf);
        match bytes_read {
            Ok(bytes_read) => {
                if bytes_read == 0 {
                    return Err(());
                }
                for byte in &buf[0..bytes_read] {
                    self.received_bytes.push_back(
                        self.packet_processing
                            .secret_cipher
                            .decrypt_u8(*byte)
                            .unwrap(),
                    );
                }

                Ok(())
            }
            Err(e) if e.kind() == ErrorKind::WouldBlock => Ok(()),
            Err(e) => {
                panic!("{:?}", e);
            }
        }
    }

    pub async fn read_incoming_packets(&mut self) {
        match self.stage {
            Stage::Handshake => {
                self.read_packets(async |packet: C2SHandshakePackets, this: &mut Self| {
                    let C2SHandshakePackets::Intention(packet) = packet;
                    this.stage = packet.intended_stage.into_stage();

                    let mut map = TypeMap::new();
                    map.insert(Event::<ReceivePacketEvent<C2SHandshakePackets>>::new());
                    map.insert(Param::new(packet));
                    let connected_server = this.connected_server.clone();
                    tokio::spawn(async move {
                        connected_server.fire_systems(map).await;
                    });
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
                // self.play_phase().await;
            }
            Stage::Transfer => todo!("doesn't exist, this needs to be removed D:"),
        }
    }

    pub async fn write_outgoing_packets(&mut self) {
        loop {
            if self.bytes_to_send.is_empty() {
                break;
            }

            if self.bytes_to_send.len() < 100 {
                println!("Bytes actually sent: {:?}", self.bytes_to_send);
            } else {
                println!("Sent big byte vector");
            }

            self.stream.write_all(&self.bytes_to_send).await.unwrap();

            self.bytes_to_send.clear();
            tokio::task::yield_now().await;
        }
    }

    pub async fn handle_messages(&mut self) {
        loop {
            match self.receiver.try_recv() {
                Ok(message) => {
                    self.handle_message(message).await;
                    break;
                }
                Err(_err) => break,
            }
        }
    }

    pub async fn handle_message(&mut self, message: ConnectionMessage) {
        match message {
            ConnectionMessage::SetStage(stage) => {
                self.stage = stage;
            }
            ConnectionMessage::GetStage(sender) => {
                let _ = sender.send(self.stage.clone());
            }
            ConnectionMessage::SendPacket(buf) => {
                self.bytes_to_send.extend(buf.as_slice());
            }
            ConnectionMessage::GetServer(sender) => {
                let server = Server {
                    sender: self.connected_server.sender.clone(),
                };
                let _ = sender.send(server);
            }
        }
    }

    pub async fn read_packets<T: PrefixedPacketDecode + Debug, F: AsyncFnOnce(T, &mut Self)>(
        &mut self,
        f: F,
    ) {
        match self
            .packet_processing
            .decode_from_raw_queue(self.received_bytes.iter().map(|x| *x))
        {
            Ok((mut buf, consumed)) => {
                if consumed == 0 {
                    return;
                }

                let mut rv = Vec::with_capacity(consumed);
                for _ in 0..consumed {
                    rv.push(self.received_bytes.pop_front().unwrap());
                }

                match T::decode_prefixed(&mut buf) {
                    Ok(packet) => {
                        println!("Received and processing bytes: {:?}", rv);
                        f(packet, self).await;
                    }
                    Err(DecodeError::EndOfBuffer) => {
                        println!("Failed to process for EoB: {:?}", rv);
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
