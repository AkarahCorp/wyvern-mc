use std::collections::HashMap;

use voxidian_protocol::{
    registry::Registry,
    value::{Biome, DamageType, DimType, PaintingVariant, WolfVariant},
};

use crate::{
    dimension::DimensionData,
    systems::{intos::IntoSystem, parameters::SystemParameter, system::System},
    values::key::Key,
};

use super::{
    ServerData,
    dimensions::DimensionContainer,
    registries::{RegistryContainer, RegistryContainerBuilder},
};

pub struct ServerBuilder {
    systems: Vec<Box<dyn System + Send + Sync + 'static>>,
    registries: RegistryContainerBuilder,
    dimensions: DimensionContainer,
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
        };

        server.start().await;
    }
}
