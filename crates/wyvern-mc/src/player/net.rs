use std::{collections::VecDeque, fmt::Debug, io::ErrorKind, net::IpAddr};

use tokio::{net::TcpStream, sync::*};
use voxidian_protocol::packet::{c2s::handshake::C2SHandshakePackets, processing::{CompressionMode, PacketProcessing, SecretCipher}, DecodeError, PrefixedPacketDecode, Stage};

use super::message::ConnectionMessage;

pub struct ConnectionData {
    stream: TcpStream,
    #[allow(dead_code)]
    addr: IpAddr,

    received_bytes: VecDeque<u8>,

    bytes_to_send: VecDeque<u8>,

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

            bytes_to_send: VecDeque::new(),

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
            self.read_incoming_packets().await;
            self.write_outgoing_packets().await;
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

    pub async fn read_incoming_packets(&mut self) {
        match self.stage {
            Stage::Handshake => {
                self.read_packets(|packet: C2SHandshakePackets| {
                    println!("{:?}", packet);
                });
            },
            Stage::Status => todo!(),
            Stage::Login => todo!(),
            Stage::Config => todo!(),
            Stage::Play => todo!(),
            Stage::Transfer => todo!("doesn't exist, this needs to be removed D:"),
        }
    }

    pub async fn write_outgoing_packets(&mut self) {
        loop {
            if self.bytes_to_send.is_empty() {
                break;
            }
            self.bytes_to_send.make_contiguous();
            match self.stream.try_write(self.bytes_to_send.as_slices().0) {
                Ok(bytes_sent) => {
                    for _ in 0..bytes_sent {
                        self.bytes_to_send.pop_front();
                    }
                }
                Err(e) if e.kind() == ErrorKind::WouldBlock => {
                    break;
                },
                Err(e) => {
                    panic!("{:?}", e);
                }
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
            ConnectionMessage::SendPacket(buf) => {
                self.bytes_to_send.extend(buf.as_slice());
            }
            #[allow(unreachable_patterns)]
            _ => todo!("unrecognized message")
        }
    }

    pub fn read_packets<T: PrefixedPacketDecode + Debug, F: FnOnce(T)>(&mut self, f: F) {
        match self.packet_processing.decode_from_raw_queue(self.received_bytes.iter().map(|x| *x)) {
            Ok((mut buf, consumed)) => {
                if consumed == 0 {
                    return;
                }

                for _ in 0..consumed {
                    self.received_bytes.pop_front();
                }

                match T::decode_prefixed(&mut buf) {
                    Ok(packet) => f(packet),
                    Err(DecodeError::EndOfBuffer) => {},
                    Err(e) => {
                        panic!("{:?}", e);
                    }
                }
            }
            Err(DecodeError::EndOfBuffer) => {},
            Err(e) => {
                panic!("err: {:?}", e);
            }
        }
    }
}