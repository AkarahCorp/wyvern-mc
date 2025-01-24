use std::{collections::HashMap, time::Instant};

use voxidian_protocol::{
    registry::Registry,
    value::{Biome, DamageType},
};

use crate::systems::{intos::IntoSystem, parameters::SystemParameter, system::System};

use super::{
    data::ServerData, dimensions::DimensionContainer, registries::RegistryContainerBuilder,
};

pub struct ServerBuilder {
    systems: Vec<Box<dyn System + Send + Sync + 'static>>,
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
            systems: Vec::new(),
            registries: RegistryContainerBuilder {
                damage_types: DamageType::vanilla_registry(),
                biomes: Biome::vanilla_registry(),
                wolf_variants: Registry::new(),
                painting_variants: Registry::new(),
                dimension_types: Registry::new(),
            },
            dimensions: DimensionContainer {
                dimensions: HashMap::new(),
            },
        }
    }

    pub fn add_system<I: SystemParameter, S>(&mut self, s: S)
    where
        S: IntoSystem<I>,
        <S as IntoSystem<I>>::System: Send + Sync + 'static,
    {
        self.systems.push(Box::new(s.into_system()));
    }

    pub fn modify_registries<F: FnOnce(&mut RegistryContainerBuilder)>(&mut self, f: F) {
        f(&mut self.registries)
    }

    pub async fn start(self) {
        let server = ServerData {
            connections: Vec::new(),
            systems: self.systems,
            registries: self.registries.into(),
            dimensions: self.dimensions,
            last_tick: Instant::now(),
        };

        server.start().await;
    }
}
