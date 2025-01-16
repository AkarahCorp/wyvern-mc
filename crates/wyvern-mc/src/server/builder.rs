use voxidian_protocol::{registry::Registry, value::{Biome, DamageType}};

use crate::systems::{intos::IntoSystem, parameters::SystemParameter, system::System};

use super::{registries::{RegistryContainer, RegistryContainerBuilder}, ServerData};

pub struct ServerBuilder {
    systems: Vec<Box<dyn System + Send + Sync + 'static>>,
    registries: RegistryContainerBuilder
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
                dimension_types: Registry::new()
            }
        }
    }

    pub fn add_system<I: SystemParameter, S>(&mut self, s: S)
    where 
        S: IntoSystem<I>,
        <S as IntoSystem<I>>::System: Send + Sync + 'static {
        self.systems.push(Box::new(s.into_system()));
    }

    pub async fn start(self) {
        let server = ServerData {
            connections: Vec::new(),
            systems: self.systems,
            registries: self.registries.into()
        };

        server.start().await;
    }
}