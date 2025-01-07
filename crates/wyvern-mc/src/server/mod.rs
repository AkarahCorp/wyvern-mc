use std::{collections::VecDeque, net::{Ipv4Addr, SocketAddrV4}};

use tokio::net::TcpListener;
use voxidian_protocol::packet::processing::{CompressionMode, PacketProcessing, SecretCipher};

use crate::player::net::RawConnection;

pub mod builder;

pub struct Server {
    #[allow(unused)]
    connections: Vec<RawConnection>
}

impl Server {
    pub async fn start(mut self) {
        let listener = TcpListener::bind(
            SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 25565)).await.unwrap();

        println!("Server now listening on 127.0.0.1:25565");
        loop {
            println!("iter");
            let new_client = listener.accept().await;
            match new_client {
                Ok((stream, addr)) => {
                    println!("Accepted new client: {:?}", addr);
                    let raw = RawConnection { 
                        stream, 
                        addr: addr.ip(), 
                        removed: false,
                        received_bytes: VecDeque::new(),
                        packet_processing: PacketProcessing {
                            secret_cipher: SecretCipher::no_cipher(),
                            compression: CompressionMode::None,
                        }
                    };
                    self.connections.push(raw);
                },
                Err(_err) => {},
            }
            self.connections.retain(|x| !x.removed);

            for connection in &mut self.connections {
                connection.event_loop().await;
            }
        }
    }
}