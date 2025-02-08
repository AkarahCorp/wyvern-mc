use voxidian_protocol::value::{EntityMetadata, Uuid};

use crate::values::{Key, Vec2, Vec3};

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

    pub async fn teleport(&mut self, position: Vec3<f64>) {
        self.dimension
            .set_entity_position(
                self.uuid,
                position,
                self.dimension
                    .get_entity_position(self.uuid)
                    .await
                    .unwrap()
                    .1,
            )
            .await;
    }
}

pub struct EntityType;

#[allow(unused)]
pub struct EntityData {
    pub(crate) entity_type: Key<EntityType>,
    pub(crate) uuid: Uuid,
    pub(crate) id: i32,

    pub(crate) position: Vec3<f64>,
    pub(crate) heading: Vec2<f32>,

    pub(crate) metadata: EntityMetadata,
}
