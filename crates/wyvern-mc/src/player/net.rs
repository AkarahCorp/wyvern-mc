use std::{collections::VecDeque, io::ErrorKind, net::IpAddr};

use tokio::net::TcpStream;
use voxidian_protocol::packet::processing::{CompressionMode, PacketProcessing, SecretCipher};

pub struct RawConnection {
    stream: TcpStream,
    addr: IpAddr,

    received_bytes: VecDeque<u8>,
    packet_processing: PacketProcessing
}

impl RawConnection {
    pub async fn execute_connection(
        stream: TcpStream,
        addr: IpAddr
    ) {
        let mut conn = RawConnection {
            stream,
            addr,
            received_bytes: VecDeque::new(),
            packet_processing: PacketProcessing {
                secret_cipher: SecretCipher::no_cipher(),
                compression: CompressionMode::None,
            },
        };

        conn.event_loop().await;
    }

    pub async fn event_loop(&mut self) {
        loop {
            let result = self.handle_incoming_bytes().await;
            if let Err(_) = result {
                break;
            }
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
}