use std::{
    net::{Ipv4Addr, SocketAddrV4},
    sync::Arc,
    time::{Duration, Instant},
};

use dimensions::DimensionContainer;
use registries::RegistryContainer;
use wyvern_actors::Actor;
use wyvern_actors_macros::{actor, message};

use crate::{
    dimension::{Dimension, DimensionData},
    player::{ConnectionData, ConnectionWithSignal, Player},
    systems::{
        events::ServerTickEvent,
        parameters::{Event, Param},
        system::System,
        typemap::TypeMap,
    },
    values::Key,
};

mod builder;
pub use builder::*;
pub mod dimensions;
pub mod registries;

use tokio::{net::TcpListener, sync::mpsc::Sender};

#[actor(Server, ServerMessage)]
pub struct ServerData {
    pub(crate) connections: Vec<ConnectionWithSignal>,
    pub(crate) systems: Vec<Box<dyn System + Send + Sync + 'static>>,
    pub(crate) registries: Arc<RegistryContainer>,
    pub(crate) dimensions: DimensionContainer,
    pub(crate) last_tick: Instant,
    pub(crate) sender: Sender<ServerMessage>,
}

#[message(Server, ServerMessage)]
impl ServerData {
    #[FireSystems]
    pub async fn fire_systems(&mut self, mut parameters: TypeMap) {
        for system in &mut self.systems {
            let server = Server {
                sender: self.sender.clone(),
            };
            if let Some(s) = system.run(&mut parameters, server) {
                s.await;
            }
        }
    }

    #[SpawnConnectionInternal]
    pub async fn spawn_connection_internal(&mut self, conn: ConnectionWithSignal) {
        self.connections.push(conn);
    }

    #[GetRegistries]
    pub async fn registries(&self) -> Arc<RegistryContainer> {
        self.registries.clone()
    }

    #[GetDimension]
    pub async fn dimension(&self, key: Key<Dimension>) -> Option<Dimension> {
        self.dimensions.get(&key).map(|dim| Dimension {
            sender: dim.sender.clone(),
        })
    }

    #[CreateDimension]
    pub async fn create_dimension(&mut self, name: Key<Dimension>) -> Dimension {
        let mut root_dim = DimensionData::new(
            unsafe { name.clone().retype() },
            Server {
                sender: self.sender.clone(),
            },
            Key::new("minecraft", "overworld"),
        );

        let dim = Dimension {
            sender: root_dim.sender.clone(),
        };
        self.dimensions.insert(name, dim.clone());
        tokio::spawn(async move {
            loop {
                root_dim.handle_messages().await;
            }
        });
        dim
    }

    #[GetConnections]
    pub async fn connections(&self) -> Vec<Player> {
        self.connections.iter().map(|x| x.lower()).collect()
    }
}

impl ServerData {
    pub async fn start(mut self) {
        self.create_dimension(Key::new("wyvern", "root")).await;
        let snd = self.sender.clone();
        tokio::spawn(self.handle_loops(snd.clone()));
        tokio::spawn(Self::networking_loop(snd));
    }

    pub async fn handle_loops(mut self, tx: Sender<ServerMessage>) {
        loop {
            self.connections
                .retain_mut(|connection| connection._signal.try_recv().is_err());

            for system in &mut self.systems {
                let mut map = TypeMap::new();
                let server = Server { sender: tx.clone() };
                if let Some(fut) = system.run(&mut map, server) {
                    tokio::spawn(fut);
                }
            }

            self.handle_messages().await;

            let dur = Instant::now().duration_since(self.last_tick);
            if dur > Duration::from_millis(50) {
                println!("Tick! {:?}", dur);
                self.last_tick = Instant::now();

                let server = Server { sender: tx.clone() };
                let server_2 = server.clone();
                tokio::spawn(async move {
                    server
                        .fire_systems({
                            let mut map = TypeMap::new();
                            map.insert(Event::<ServerTickEvent>::new());
                            map.insert(Param::new(server_2));
                            map
                        })
                        .await;
                });
            }
        }
    }

    pub async fn networking_loop(tx: Sender<ServerMessage>) {
        let listener = TcpListener::bind(SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 25565))
            .await
            .unwrap();

        println!("Server now listening on 127.0.0.1:25565");
        loop {
            let new_client = listener.accept().await;
            match new_client {
                Ok((stream, addr)) => {
                    println!("Accepted new client: {:?}", addr);

                    let server = Server { sender: tx.clone() };
                    let signal =
                        ConnectionData::connection_channel(stream, addr.ip(), server.clone());
                    server.spawn_connection_internal(signal).await;
                }
                Err(_err) => {}
            }
        }
    }
}
