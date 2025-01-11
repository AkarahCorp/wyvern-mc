use std::{net::{Ipv4Addr, SocketAddrV4}, sync::Arc};

use message::ServerMessage;
use proxy::Server;
use tokio::{net::TcpListener, sync::mpsc::{Receiver, Sender}};

use crate::{player::{net::ConnectionData, proxy::ConnectionWithSignal}, systems::{system::System, typemap::TypeMap}};

pub mod builder;
pub mod proxy;
pub mod message;

pub struct ServerData {
    connections: Vec<ConnectionWithSignal>,
    systems: Vec<Box<dyn System + Send + Sync + 'static>>
}

impl ServerData {
    pub async fn start(self) {
        let (tx, rx) = tokio::sync::mpsc::channel::<ServerMessage>(16);
        tokio::spawn(self.handle_loops(tx.clone(), rx));
        tokio::spawn(Self::networking_loop(tx));

        loop {}
    }

    pub async fn handle_loops(mut self, tx: Sender<ServerMessage>, mut rx: Receiver<ServerMessage>) {
        loop {
            self.connections
                .retain_mut(|connection| {
                    connection._signal.try_recv().is_err()
                });

            for system in &mut self.systems {
                let mut map = TypeMap::new();
                let server = Server { sender: tx.clone() };
                if let Some(fut) = system.run(&mut map, server) {
                    tokio::spawn(fut);
                }
            }


            if let Ok(msg) = rx.try_recv() {
                match msg {
                    ServerMessage::SpawnConnection(connection_with_signal) => {
                        self.connections.push(connection_with_signal);
                    },
                    ServerMessage::FireSystems(mut parameters) => {
                        for system in &mut self.systems {
                            let server = Server { sender: tx.clone() };
                            if let Some(fut) = system.run(&mut parameters, server) {
                                tokio::spawn(fut);
                            }
                        }
                    }
                }
            };

            
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

                    
                    let server = Server { sender: tx.clone() };
                    let (messenger, signal) = ConnectionData::connection_channel(
                        stream, 
                        addr.ip(),
                        server
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