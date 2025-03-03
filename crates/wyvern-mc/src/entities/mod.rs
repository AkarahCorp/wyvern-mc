use voxidian_protocol::value::{EntityMetadata, Uuid};

use crate::{
    actors::ActorResult,
    dimension::Dimension,
    values::{Id, Vec2, Vec3},
};

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

    pub fn remove(&self) -> ActorResult<()> {
        self.dimension.remove_entity(self.uuid)?;
        Ok(())
    }

    pub fn entity_id(&self) -> ActorResult<i32> {
        self.dimension.entity_id(self.uuid)
    }

    pub fn entity_type(&self) -> ActorResult<Id> {
        self.dimension.entity_type(self.uuid)
    }

    pub fn position(&self) -> ActorResult<(Vec3<f64>, Vec2<f32>)> {
        self.dimension.entity_pos(self.uuid)
    }

    pub fn teleport(&mut self, position: Vec3<f64>) -> ActorResult<()> {
        self.dimension.teleport_entity(self.uuid, position)
    }

    pub fn rotate(&mut self, heading: Vec2<f32>) -> ActorResult<()> {
        self.dimension.rotate_entity(self.uuid, heading)
    }
}

pub struct EntityType;

#[allow(unused)]
#[derive(Debug, Clone)]
pub struct EntityData {
    pub(crate) entity_type: Id,
    pub(crate) uuid: Uuid,
    pub(crate) id: i32,

    pub(crate) position: Vec3<f64>,
    pub(crate) heading: Vec2<f32>,

    pub(crate) metadata: EntityMetadata,
}

pub struct Entities;
wyvern_macros::generate_entity_types!();
