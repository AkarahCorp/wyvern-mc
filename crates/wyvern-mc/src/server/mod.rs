use std::{
    net::{Ipv4Addr, SocketAddrV4},
    sync::Arc,
    time::{Duration, Instant},
};

use crate::{actor, message};
use crate::{actors::Actor, runtime::Runtime};
use async_net::TcpListener;
use dimensions::DimensionContainer;
use flume::Sender;
use registries::RegistryContainer;

use crate::{
    dimension::{Dimension, DimensionData},
    events::{DimensionCreateEvent, Event, EventBus, ServerStartEvent, ServerTickEvent},
    player::{ConnectionData, ConnectionWithSignal, Player},
    values::Key,
};

mod builder;
pub use builder::*;
pub mod dimensions;
pub mod registries;

#[actor(Server, ServerMessage)]
pub(crate) struct ServerData {
    pub(crate) connections: Vec<ConnectionWithSignal>,
    pub(crate) registries: Arc<RegistryContainer>,
    pub(crate) dimensions: DimensionContainer,
    pub(crate) last_tick: Instant,
    pub(crate) sender: Sender<ServerMessage>,
    pub(crate) events: Arc<EventBus>,
    pub(crate) last_entity_id: i32,
}

impl Server {
    pub fn spawn_event<E: Event + Send + 'static>(&self, event: E) {
        let server = self.clone();
        Runtime::spawn(async move {
            event.dispatch(server.event_bus().await);
        });
    }

    pub async fn spawn_event_blocking<E: Event + Send + 'static>(&self, event: E) {
        let server = self.clone();
        let bus = server.event_bus().await;
        event.dispatch(bus);
    }
}

#[message(Server, ServerMessage)]
impl ServerData {
    #[NewEntityId]
    pub async fn get_entity_id(&mut self) -> i32 {
        self.last_entity_id += 1;
        log::debug!("New entity id produced: {:?}", self.last_entity_id);
        self.last_entity_id
    }

    #[GetEventBus]
    pub async fn event_bus(&mut self) -> Arc<EventBus> {
        self.events.clone()
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

    #[GetAllDimensions]
    pub async fn get_all_dimensions(&self) -> Vec<Dimension> {
        self.dimensions.dimensions().cloned().collect()
    }

    #[CreateDimension]
    pub async fn create_dimension(&mut self, name: Key<Dimension>) -> Dimension {
        log::debug!("Creating new dimension: {:?}", name);
        let mut root_dim = DimensionData::new(
            name.clone().retype(),
            Server {
                sender: self.sender.clone(),
            },
            Key::new("minecraft", "overworld"),
        );

        let dim = Dimension {
            sender: root_dim.sender.clone(),
        };
        self.dimensions.insert(name, dim.clone());
        Runtime::spawn(async move {
            loop {
                root_dim.handle_messages().await;
            }
        });

        let dim_clone = dim.clone();
        let server_clone = Server {
            sender: self.sender.clone(),
        };
        server_clone.spawn_event(DimensionCreateEvent {
            dimension: dim_clone,
            server: server_clone.clone(),
        });
        futures_lite::future::yield_now().await;
        futures_lite::future::yield_now().await;
        futures_lite::future::yield_now().await;
        dim
    }

    #[GetConnections]
    pub async fn connections(&self) -> Vec<Player> {
        self.connections.iter().map(|x| x.lower()).collect()
    }
}

impl ServerData {
    pub async fn start(mut self) {
        log::info!("A server is starting!");
        self.create_dimension(Key::new("wyvern", "root")).await;
        let snd = Server {
            sender: self.sender.clone(),
        };
        let snd_clone = snd.clone();
        Runtime::spawn(async move {
            snd_clone.spawn_event(ServerStartEvent {
                server: snd_clone.clone(),
            });
        });
        Runtime::spawn(self.handle_loops(snd.clone()));
        Runtime::spawn(Self::networking_loop(snd));
    }

    pub async fn handle_loops(mut self, server: Server) {
        loop {
            self.connections
                .retain_mut(|connection| connection._signal.try_recv().is_err());

            self.handle_messages().await;

            let dur = Instant::now().duration_since(self.last_tick);
            if dur > Duration::from_millis(50) {
                self.last_tick = Instant::now();

                server.spawn_event(ServerTickEvent {
                    server: server.clone(),
                });
            }
        }
    }

    pub async fn networking_loop(server: Server) {
        let listener = TcpListener::bind(SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 25565))
            .await
            .unwrap();

        log::info!("A server is now listening on: 127.0.0.1:25565");
        loop {
            let new_client = listener.accept().await;
            match new_client {
                Ok((stream, addr)) => {
                    log::info!("Accepted new client: {:?}", addr);
                    let signal =
                        ConnectionData::connection_channel(stream, addr.ip(), server.clone());
                    server.spawn_connection_internal(signal).await;
                }
                Err(_err) => {}
            }
        }
    }
}
