use std::{collections::HashMap, pin::Pin, sync::Arc, time::Instant};

use voxidian_protocol::{
    registry::Registry,
    value::{Biome, DamageType, EntityType},
};

use crate::events::{Event, EventBus};

use super::{ServerData, dimensions::DimensionContainer, registries::RegistryContainerBuilder};

pub struct ServerBuilder {
    events: EventBus,
    registries: RegistryContainerBuilder,
    dimensions: DimensionContainer,
}

impl Default for ServerBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl ServerBuilder {
    pub fn new() -> ServerBuilder {
        ServerBuilder {
            events: EventBus::default(),
            registries: RegistryContainerBuilder {
                damage_types: DamageType::vanilla_registry().into(),
                biomes: Biome::vanilla_registry().into(),
                wolf_variants: Registry::new().into(),
                painting_variants: Registry::new().into(),
                dimension_types: Registry::new().into(),
                entity_types: EntityType::vanilla_registry().into(),
            },
            dimensions: DimensionContainer {
                dimensions: HashMap::new(),
            },
        }
    }

    pub fn on_event<E: Event + 'static, F>(&mut self, f: fn(E) -> F)
    where
        F: Future<Output = ()> + Send + 'static,
    {
        let handler = Box::new(move |event: E| {
            Box::pin(f(event)) as Pin<Box<dyn Future<Output = ()> + Send>>
        })
            as Box<dyn Fn(E) -> Pin<Box<dyn Future<Output = ()> + Send>> + Send + Sync>;
        E::add_handler(&mut self.events, handler);
    }

    pub fn modify_registries<F: FnOnce(&mut RegistryContainerBuilder)>(&mut self, f: F) {
        f(&mut self.registries)
    }

    pub async fn start(self) {
        let chan = flume::unbounded();
        let server = ServerData {
            connections: Vec::new(),
            registries: Arc::new(self.registries.into()),
            dimensions: self.dimensions,
            last_tick: Instant::now(),

            sender: chan.0,
            receiver: chan.1,
            events: Arc::new(self.events),

            last_entity_id: 0,
        };

        server.start().await;
    }
}
