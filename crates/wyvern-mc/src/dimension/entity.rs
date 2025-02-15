use voxidian_protocol::value::{EntityMetadata, Uuid};

use crate::{
    actors::ActorResult,
    values::{Key, Vec2, Vec3},
};

use super::Dimension;

#[derive(Debug)]
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

    pub async fn remove(&self) -> ActorResult<()> {
        self.dimension.remove_entity(self.uuid).await?;
        Ok(())
    }

    pub async fn entity_id(&self) -> ActorResult<i32> {
        self.dimension.entity_id(self.uuid).await
    }

    pub async fn entity_type(&self) -> ActorResult<Key<EntityType>> {
        self.dimension.entity_type(self.uuid).await
    }

    pub async fn position(&self) -> ActorResult<(Vec3<f64>, Vec2<f32>)> {
        self.dimension.entity_pos(self.uuid).await
    }

    pub async fn teleport(&mut self, position: Vec3<f64>) -> ActorResult<()> {
        self.dimension.teleport_entity(self.uuid, position).await
    }

    pub async fn rotate(&mut self, heading: Vec2<f32>) -> ActorResult<()> {
        self.dimension.rotate_entity(self.uuid, heading).await
    }
}

pub struct EntityType;

#[allow(unused)]
#[derive(Debug, Clone)]
pub struct EntityData {
    pub(crate) entity_type: Key<EntityType>,
    pub(crate) uuid: Uuid,
    pub(crate) id: i32,

    pub(crate) position: Vec3<f64>,
    pub(crate) heading: Vec2<f32>,

    pub(crate) metadata: EntityMetadata,
}

pub struct Entities;
wyvern_macros::generate_entity_types!();
