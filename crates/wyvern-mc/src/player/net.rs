use std::{collections::VecDeque, fmt::Debug, io::ErrorKind, net::IpAddr};

use tokio::{io::AsyncWriteExt, net::TcpStream, sync::*};
use voxidian_protocol::packet::{c2s::{config::C2SConfigPackets, handshake::C2SHandshakePackets, login::C2SLoginPackets, play::C2SPlayPackets, status::C2SStatusPackets}, processing::{CompressionMode, PacketProcessing, SecretCipher}, DecodeError, PacketBuf, PrefixedPacketDecode, PrefixedPacketEncode, Stage};

use super::message::ConnectionMessage;

pub struct ConnectionData {
    stream: TcpStream,
    addr: IpAddr,

    received_bytes: VecDeque<u8>,
    packet_processing: PacketProcessing,

    receiver: mpsc::Receiver<ConnectionMessage>,
    signal: mpsc::Sender<ConnectionStoppedSignal>,

    stage: Stage
}

pub struct ConnectionStoppedSignal;

impl ConnectionData {
    pub fn connection_channel(
        stream: TcpStream,
        addr: IpAddr
    ) -> (mpsc::Sender<ConnectionMessage>, mpsc::Receiver<ConnectionStoppedSignal>) {
        let (signal_tx, signal_rx) = mpsc::channel(1);
        let (data_tx, data_rx) = mpsc::channel(256);

        tokio::spawn(ConnectionData::execute_connection(stream, addr, data_rx, signal_tx));

        (data_tx, signal_rx)
    }

    pub async fn execute_connection(
        stream: TcpStream,
        addr: IpAddr,
        receiver: mpsc::Receiver<ConnectionMessage>,
        signal: mpsc::Sender<ConnectionStoppedSignal>
    ) {
        let mut conn = ConnectionData {
            stream,
            addr,
            received_bytes: VecDeque::new(),
            packet_processing: PacketProcessing {
                secret_cipher: SecretCipher::no_cipher(),
                compression: CompressionMode::None,
            },

            receiver,
            signal,

            stage: Stage::Handshake
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
                            .unwrap()
                    );
                }
                println!("recv: {:?}", self.received_bytes);

                Ok(())
            }
            Err(e) if e.kind() == ErrorKind::WouldBlock => {
                Ok(())
            }
            Err(e) => {
                panic!("{:?}", e);
            }
        }
    }

    pub async fn handle_messages(&mut self) {
        loop {
            match self.receiver.try_recv() {
                Ok(message) => {
                    self.handle_message(message).await;
                    break;
                },
                Err(_err) => break,
            }
        }
    }

    pub async fn handle_message(&mut self, message: ConnectionMessage) {
        match message {
            ConnectionMessage::SetStage(stage) => {
                self.stage = stage;
            },
            ConnectionMessage::GetStage(sender) => {
                sender.send(self.stage.clone()).unwrap();
            },
            ConnectionMessage::ReadHandshakingPacket(sender) => {
                sender.send(self.read_packets::<C2SHandshakePackets>()).unwrap();
            }
            ConnectionMessage::ReadStatusPacket(sender) => {
                sender.send(self.read_packets::<C2SStatusPackets>()).unwrap();
            }
            ConnectionMessage::SendStatusPacket(packet) => {
                let mut buf = PacketBuf::new();
                packet.encode_prefixed(&mut buf).unwrap();
                self.stream.write_all(buf.as_slice()).await.unwrap();
            }
            ConnectionMessage::ReadLoginPacket(sender) => {
                sender.send(self.read_packets::<C2SLoginPackets>()).unwrap();
            }
            ConnectionMessage::SendLoginPacket(packet) => {
                let mut buf = PacketBuf::new();
                packet.encode_prefixed(&mut buf).unwrap();
                self.stream.write_all(buf.as_slice()).await.unwrap();
            }
            ConnectionMessage::ReadConfigPacket(sender) => {
                sender.send(self.read_packets::<C2SConfigPackets>()).unwrap();
            }
            ConnectionMessage::SendConfigPacket(packet) => {
                let mut buf = PacketBuf::new();
                packet.encode_prefixed(&mut buf).unwrap();
                self.stream.write_all(buf.as_slice()).await.unwrap();
            }
            ConnectionMessage::ReadPlayPacket(sender) => {
                sender.send(self.read_packets::<C2SPlayPackets>()).unwrap();
            }
            ConnectionMessage::SendPlayPacket(packet) => {
                let mut buf = PacketBuf::new();
                packet.encode_prefixed(&mut buf).unwrap();
                self.stream.write_all(buf.as_slice()).await.unwrap();
            }
            _ => todo!("unrecognized message")
        }
    }

    pub fn read_packets<T: PrefixedPacketDecode + Debug>(&mut self) -> Option<T> {
        
        let output = match self.packet_processing.decode_from_raw_queue(self.received_bytes.iter().map(|x| *x)) {
            Ok((mut buf, consumed)) => {
                if consumed == 0 {
                    return None;
                }

                for _ in 0..consumed {
                    self.received_bytes.pop_front();
                }

                match T::decode_prefixed(&mut buf) {
                    Ok(packet) => Some(packet),
                    Err(DecodeError::EndOfBuffer) => None,
                    Err(e) => {
                        panic!("{:?}", e);
                    }
                }
            }
            Err(DecodeError::EndOfBuffer) => None,
            Err(e) => {
                panic!("err: {:?}", e);
            }
        };

        output
    }
}