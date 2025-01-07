use std::{net::{Ipv4Addr, SocketAddrV4}, sync::Arc};

use tokio::net::TcpListener;
use voxidian_protocol::packet::{c2s::handshake::C2SHandshakePackets, Stage};

use crate::player::{net::ConnectionData, proxy::ConnectionWithSignal};

pub mod builder;

pub struct Server {
    connections: Vec<ConnectionWithSignal>
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

                    let (messenger, signal) = ConnectionData::connection_channel(
                        stream, 
                        addr.ip()
                    );
                    let proxy = ConnectionWithSignal {
                        messenger: Arc::new(messenger),
                        signal,
                    };
                    let lowered = proxy.lower();
                    tokio::spawn(async move {
                        loop {
                            match lowered.get_stage().await {
                                Stage::Handshake => {
                                    let Some(packet) = lowered.read_handshaking_packet().await else {
                                        tokio::task::yield_now().await;
                                        continue;
                                    };
                                    println!("handshake packet: {:?}", packet);
                                }
                                Stage::Status => {
                                    let Some(packet) = lowered.read_status_packet().await else {
                                        tokio::task::yield_now().await;
                                        continue;
                                    };
                                    println!("status packet: {:?}", packet);
                                }
                                Stage::Login => {
                                    let Some(packet) = lowered.read_login_packet().await else {
                                        tokio::task::yield_now().await;
                                        continue;
                                    };
                                    println!("login packet: {:?}", packet);
                                }
                                _ => {}
                            }
                        }
                    });
                    self.connections.push(proxy);
                },
                Err(_err) => {},
            }
        }
    }
}