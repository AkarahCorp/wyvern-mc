use std::sync::Arc;

use dyn_clone::clone_box;
use voxidian_protocol::value::{EntityMetadata, MetadataEntry, Uuid};
use wyvern_components::{ComponentElement, DataComponentMap, DataComponentType};

use crate::{
    actors::{ActorError, ActorResult},
    dimension::Dimension,
};
use wyvern_values::{Id, id};

mod components;
pub use components::*;
mod attributes;
pub use attributes::*;
mod update;
pub use update::*;

#[derive(Clone, Debug)]
pub struct Entity {
    pub(crate) dimension: Dimension,
    pub(crate) uuid: Uuid,
}

impl Entity {
    pub fn uuid(&self) -> &Uuid {
        &self.uuid
    }

    pub fn dimension(&self) -> &Dimension {
        &self.dimension
    }

    pub fn remove(&self) -> ActorResult<()> {
        self.dimension.remove_entity(self.uuid)?;
        Ok(())
    }

    pub fn get<T: ComponentElement>(&self, component: DataComponentType<T>) -> ActorResult<T> {
        let component = self
            .dimension
            .get_entity_component_unchecked(self.uuid, component.into_name())?;

        ((*component).as_any().downcast_ref::<T>())
            .map(|x| clone_box(x))
            .map(|x| *x)
            .ok_or(ActorError::ComponentNotFound)
    }

    pub fn set<T: ComponentElement>(
        &self,
        component: DataComponentType<T>,
        value: T,
    ) -> ActorResult<()> {
        self.dimension.set_entity_component_unchecked(
            self.uuid,
            component.into_name(),
            Arc::new(value),
        )
    }

    pub fn generate_metadata(&self) -> ActorResult<EntityMetadata> {
        let mut meta = EntityMetadata::new();
        if self.get(EntityComponents::ENTITY_TYPE)? == id![minecraft:player] {
            meta.insert_raw_entry(17, MetadataEntry::Byte(255));
        }
        Ok(meta)
    }
}

pub struct EntityType;

#[allow(unused)]
#[derive(Debug, Clone)]
pub struct EntityData {
    pub(crate) last_components: DataComponentMap,
    pub(crate) components: DataComponentMap,
}

pub struct Entities;
wyvern_macros::generate_entity_types!();
