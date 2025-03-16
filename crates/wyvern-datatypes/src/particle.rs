use voxidian_protocol::{
    registry::RegEntry,
    value::{ParticleData, ParticleInstance, ParticleType},
};
use wyvern_components::{DataComponentHolder, DataComponentMap, DataComponentType};
use wyvern_values::{Id, id};

pub struct Particle {
    components: DataComponentMap,
}

impl Particle {
    pub fn new(id: Id) -> Particle {
        Particle {
            components: DataComponentMap::new().with(ParticleComponents::TYPE, id),
        }
    }
}

pub struct ParticleComponents;

impl ParticleComponents {
    pub const TYPE: DataComponentType<Id> = DataComponentType::new(id![minecraft:particle_type]);
}

impl DataComponentHolder for Particle {
    fn component_map(&self) -> &DataComponentMap {
        &self.components
    }

    fn component_map_mut(&mut self) -> &mut DataComponentMap {
        &mut self.components
    }
}

impl From<Particle> for ParticleInstance {
    fn from(value: Particle) -> Self {
        let id = value
            .components
            .get(ParticleComponents::TYPE)
            .unwrap_or(Id::empty());

        let data = ParticleData::None;
        ParticleInstance {
            base: ParticleType::vanilla_registry()
                .get_entry(&id.into())
                .unwrap_or(unsafe { RegEntry::new_unchecked(0) }),
            data,
        }
    }
}
