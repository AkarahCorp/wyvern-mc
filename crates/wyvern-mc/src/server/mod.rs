use std::{
    net::{Ipv4Addr, SocketAddrV4},
    sync::{
        Arc, Mutex, OnceLock,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};

use crate::{
    actor,
    actors::{ActorError, ActorResult},
    message,
};
use crate::{actors::Actor, runtime::Runtime};
use dimensions::DimensionContainer;
use flume::Sender;
use registries::RegistryContainer;
use voxidian_protocol::{packet::Stage, value::Uuid};

use crate::{
    dimension::{Dimension, DimensionData},
    events::{DimensionCreateEvent, Event, EventBus, ServerStartEvent, ServerTickEvent},
    player::{ConnectionData, ConnectionWithSignal, Player},
    values::Id,
};

mod builder;
pub use builder::*;
pub mod dimensions;
pub mod registries;

static SERVER_INSTANCE: OnceLock<Server> = OnceLock::new();

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
    pub fn get() -> ActorResult<Server> {
        SERVER_INSTANCE
            .get()
            .ok_or(ActorError::ActorDoesNotExist)
            .cloned()
    }

    pub fn spawn_event<E: Event + Send + 'static>(&self, event: E) -> ActorResult<()> {
        let server = self.clone();
        Runtime::spawn(move || {
            event.dispatch(server.event_bus().unwrap());
        });
        Ok(())
    }
}

#[message(Server, ServerMessage)]
impl ServerData {
    #[NewEntityId]
    pub fn new_entity_id(&mut self) -> ActorResult<i32> {
        self.last_entity_id += 1;
        log::debug!("New entity id produced: {:?}", self.last_entity_id);
        Ok(self.last_entity_id)
    }

    #[GetEventBus]
    pub fn event_bus(&mut self) -> ActorResult<Arc<EventBus>> {
        Ok(self.events.clone())
    }

    #[SpawnConnectionInternal]
    pub fn spawn_connection_internal(&mut self, conn: ConnectionWithSignal) -> ActorResult<()> {
        self.connections.push(conn);
        Ok(())
    }

    #[GetRegistries]
    pub fn registries(&self) -> ActorResult<Arc<RegistryContainer>> {
        Ok(self.registries.clone())
    }

    #[GetDimension]
    pub fn dimension(&self, key: Id) -> ActorResult<Dimension> {
        self.dimensions
            .get(&key)
            .map(|dim| Dimension {
                sender: dim.sender.clone(),
            })
            .ok_or(ActorError::IndexOutOfBounds)
    }

    #[GetAllDimensions]
    pub fn dimensions(&self) -> ActorResult<Vec<Dimension>> {
        Ok(self.dimensions.dimensions().cloned().collect())
    }

    #[CreateDimension]
    pub fn create_dimension(&mut self, name: Id) -> ActorResult<Dimension> {
        log::debug!("Creating new dimension: {:?}", name);
        let mut root_dim = DimensionData::new(
            name.clone(),
            Server {
                sender: self.sender.clone(),
            },
            Id::new("minecraft", "overworld"),
        );

        let dim = Dimension {
            sender: root_dim.sender.clone(),
        };
        self.dimensions.insert(name, dim.clone());
        Runtime::spawn(move || {
            loop {
                root_dim.handle_messages();
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

        Ok(dim)
    }

    #[GetConnections]
    pub fn connections(&self) -> ActorResult<Vec<Player>> {
        Ok(self.connections.iter().map(|x| x.lower()).collect())
    }

    #[GetPlayers]
    pub fn players(&mut self) -> ActorResult<Vec<Player>> {
        let mut vec = Vec::new();

        for conn in &self.connections {
            let stage = *conn.stage.lock().unwrap() == Stage::Play;
            let loaded = conn.is_loaded.load(Ordering::Acquire);

            if stage && loaded {
                vec.push(conn.lower());
            }
        }
        Ok(vec)
    }

    #[GetPlayerByUuid]
    pub fn player(&self, player: Uuid) -> ActorResult<Player> {
        for conn in &self.connections {
            if conn.player.uuid() == Ok(player) && conn.player.stage() == Ok(Stage::Play) {
                return Ok(conn.player.clone());
            }
        }
        Err(ActorError::BadRequest)
    }
}

impl ServerData {
    pub fn start(self) {
        log::info!("A server is starting!");
        let snd = Server {
            sender: self.sender.clone(),
        };

        SERVER_INSTANCE.set(snd.clone()).unwrap_or_else(|_| {
            log::error!("WyvernMC does not support running two servers at once. Bugs may occur.");
        });
        let snd_clone = snd.clone();
        Runtime::spawn(move || {
            let _ = snd_clone.spawn_event(ServerStartEvent {
                server: snd_clone.clone(),
            });
        });
        let snd_clone = snd.clone();
        Runtime::spawn(move || Self::networking_loop(snd_clone));
        self.handle_loops(snd);
    }

    pub fn handle_loops(mut self, server: Server) {
        loop {
            self.connections.retain_mut(|connection| {
                if connection._signal.try_recv().is_err() {
                    true
                } else {
                    log::error!("Receiving drop signal and trying to drop");
                    log::error!("{:?}", connection.player.sender.sender_count());
                    false
                }
            });

            self.handle_messages();
            let dur = Instant::now().duration_since(self.last_tick);
            if dur > Duration::from_millis(50) {
                self.last_tick = Instant::now();

                let _ = server.spawn_event(ServerTickEvent {
                    server: server.clone(),
                });
            }
        }
    }

    pub fn networking_loop(server: Server) {
        let listener =
            std::net::TcpListener::bind(SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 25565))
                .unwrap();

        log::info!("A server is now listening on: 127.0.0.1:25565");
        loop {
            let new_client = listener.accept();
            match new_client {
                Ok((stream, addr)) => {
                    log::info!("Accepted new client: {:?}", addr);
                    let stage = Arc::new(Mutex::new(Stage::Handshake));
                    let is_loaded = Arc::new(AtomicBool::new(false));
                    let signal = ConnectionData::connection_channel(
                        stream,
                        addr.ip(),
                        server.clone(),
                        stage,
                        is_loaded,
                    );
                    let _ = server.spawn_connection_internal(signal);
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
