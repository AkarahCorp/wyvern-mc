use std::{net::{Ipv4Addr, SocketAddrV4}, sync::Arc};

use message::ServerMessage;
use tokio::{net::TcpListener, sync::mpsc::{Receiver, Sender}};

use crate::player::{net::ConnectionData, proxy::ConnectionWithSignal};

pub mod builder;
pub mod proxy;
pub mod message;

pub struct ServerData {
    connections: Vec<ConnectionWithSignal>
}

impl ServerData {
    pub async fn start(self) {
        let (tx, rx) = tokio::sync::mpsc::channel::<ServerMessage>(16);
        tokio::spawn(self.handle_messages(rx));
        tokio::spawn(Self::networking_loop(tx));
    }

    pub async fn handle_messages(mut self, mut rx: Receiver<ServerMessage>) {
        loop {
            let Some(msg) = rx.recv().await else {
                continue;
            };

            match msg {
                ServerMessage::SpawnConnection(connection_with_signal) => {
                    self.connections.push(connection_with_signal);
                },
            }
        }
    }

    pub async fn networking_loop(tx: Sender<ServerMessage>) {
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
                        _signal: signal,
                    };
                    let _lowered = proxy.lower();
                    tx.send(ServerMessage::SpawnConnection(proxy)).await.unwrap();
                },
                Err(_err) => {},
            }
        }
    }
}