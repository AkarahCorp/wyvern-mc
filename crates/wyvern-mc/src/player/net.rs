use std::{collections::VecDeque, io::ErrorKind, net::IpAddr};

use tokio::net::TcpStream;
use voxidian_protocol::packet::processing::PacketProcessing;

pub struct RawConnection {
    pub(crate) stream: TcpStream,
    #[allow(unused)]
    pub(crate) addr: IpAddr,
    pub(crate) removed: bool,

    pub(crate) received_bytes: VecDeque<u8>,
    pub(crate) packet_processing: PacketProcessing
}

impl RawConnection {
    pub async fn event_loop(&mut self) {
        self.handle_incoming_bytes().await;
        self.write_outgoing_packets().await;
    }

    pub async fn handle_incoming_bytes(&mut self) {
        let mut buf = [0; 256];
        let bytes_read = self.stream.try_read(&mut buf);
        match bytes_read {
            Ok(bytes_read) => {
                if bytes_read == 0 {
                    self.removed = true;
                    return;
                }
                for byte in &buf[0..bytes_read] {
                    self.received_bytes.push_back(
                        self
                            .packet_processing
                            .secret_cipher
                            .decrypt_u8(*byte)
                            .unwrap()
                    );
                }
                println!("recv: {:?}", self.received_bytes);
            }
            Err(e) if e.kind() == ErrorKind::WouldBlock => {

            }
            Err(e) => {
                panic!("{:?}", e);
            }
        }
    }

    pub async fn handle_incoming_packets(&mut self) {

    }

    pub async fn write_outgoing_packets(&mut self) {

    }
}