use std::{
    net::{Ipv4Addr, SocketAddrV4},
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use crate::{
    actor,
    actors::{ActorError, ActorResult},
    message,
};
use crate::{actors::Actor, runtime::Runtime};
use async_net::TcpListener;
use dimensions::DimensionContainer;
use flume::Sender;
use registries::RegistryContainer;
use voxidian_protocol::{packet::Stage, value::Uuid};

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
    pub fn spawn_event<E: Event + Send + 'static>(&self, event: E) -> ActorResult<()> {
        let server = self.clone();
        Runtime::spawn(async move {
            event.dispatch(server.event_bus().await.unwrap());
        });
        Ok(())
    }

    pub async fn spawn_event_blocking<E: Event + Send + 'static>(
        &self,
        event: E,
    ) -> ActorResult<()> {
        let server = self.clone();
        let bus = server.event_bus().await?;
        event.dispatch(bus);
        Ok(())
    }
}

#[message(Server, ServerMessage)]
impl ServerData {
    #[NewEntityId]
    pub async fn new_entity_id(&mut self) -> ActorResult<i32> {
        self.last_entity_id += 1;
        log::debug!("New entity id produced: {:?}", self.last_entity_id);
        Ok(self.last_entity_id)
    }

    #[GetEventBus]
    pub async fn event_bus(&mut self) -> ActorResult<Arc<EventBus>> {
        Ok(self.events.clone())
    }

    #[SpawnConnectionInternal]
    pub async fn spawn_connection_internal(
        &mut self,
        conn: ConnectionWithSignal,
    ) -> ActorResult<()> {
        self.connections.push(conn);
        Ok(())
    }

    #[GetRegistries]
    pub async fn registries(&self) -> ActorResult<Arc<RegistryContainer>> {
        Ok(self.registries.clone())
    }

    #[GetDimension]
    pub async fn dimension(&self, key: Key<Dimension>) -> ActorResult<Dimension> {
        self.dimensions
            .get(&key)
            .map(|dim| Dimension {
                sender: dim.sender.clone(),
            })
            .ok_or(ActorError::IndexOutOfBounds)
    }

    #[GetAllDimensions]
    pub async fn dimensions(&self) -> ActorResult<Vec<Dimension>> {
        Ok(self.dimensions.dimensions().cloned().collect())
    }

    #[CreateDimension]
    pub async fn create_dimension(&mut self, name: Key<Dimension>) -> ActorResult<Dimension> {
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
        let _ = server_clone.spawn_event(DimensionCreateEvent {
            dimension: dim_clone,
            server: server_clone.clone(),
        });

        futures_lite::future::yield_now().await;
        Ok(dim)
    }

    #[GetConnections]
    pub async fn connections(&self) -> Vec<Player> {
        self.connections.iter().map(|x| x.lower()).collect()
    }

    #[GetPlayers]
    pub async fn players(&self) -> Vec<Player> {
        let mut vec = Vec::new();
        for conn in &self.connections {
            if *conn.stage.lock().unwrap() == Stage::Play {
                vec.push(conn.lower());
            }
        }
        vec
    }

    #[GetPlayerByUuid]
    pub async fn player(&self, player: Uuid) -> ActorResult<Player> {
        for conn in &self.connections {
            if conn.player.uuid().await == Ok(player) {
                return Ok(conn.player.clone());
            }
        }
        Err(ActorError::BadRequest)
    }
}

impl ServerData {
    pub async fn start(self) {
        log::info!("A server is starting!");
        let snd = Server {
            sender: self.sender.clone(),
        };
        let snd_clone = snd.clone();
        Runtime::spawn(async move {
            let _ = snd_clone.spawn_event(ServerStartEvent {
                server: snd_clone.clone(),
            });
        });
        Runtime::spawn(Self::networking_loop(snd.clone()));
        self.handle_loops(snd).await;
    }

    pub async fn handle_loops(mut self, server: Server) {
        loop {
            self.connections
                .retain_mut(|connection| connection._signal.try_recv().is_err());

            self.handle_messages().await;

            let dur = Instant::now().duration_since(self.last_tick);
            if dur > Duration::from_millis(50) {
                self.last_tick = Instant::now();

                let _ = server.spawn_event(ServerTickEvent {
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
                    let stage = Arc::new(Mutex::new(Stage::Handshake));
                    let signal = ConnectionData::connection_channel(
                        stream,
                        addr.ip(),
                        server.clone(),
                        stage,
                    );
                    let _ = server.spawn_connection_internal(signal).await;
                }
                Err(_err) => {}
            }
        }
    }
}

impl Server {
    pub fn builder() -> ServerBuilder {
        ServerBuilder::new()
    }
}
