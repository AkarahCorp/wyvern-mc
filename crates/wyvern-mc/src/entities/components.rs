use voxidian_protocol::value::Uuid;

use crate::{
    components::{DataComponentHolder, DataComponentType},
    id,
    values::{Id, Vec2, Vec3},
};

use super::EntityData;

pub struct EntityComponents;

impl EntityComponents {
    pub const ENTITY_TYPE: DataComponentType<Id> =
        DataComponentType::new(id![minecraft:entity_type]);
    pub const POSITION: DataComponentType<Vec3<f64>> =
        DataComponentType::new(id![minecraft:position]);
    pub const DIRECTION: DataComponentType<Vec2<f32>> =
        DataComponentType::new(id![minecraft:direction]);
    pub const UUID: DataComponentType<Uuid> = DataComponentType::new(id![minecraft:uuid]);
    pub const ENTITY_ID: DataComponentType<i32> = DataComponentType::new(id![minecraft:entity_id]);
}

impl DataComponentHolder for EntityData {
    fn component_map(&self) -> &crate::components::DataComponentMap {
        &self.components
    }

    fn component_map_mut(&mut self) -> &mut crate::components::DataComponentMap {
        &mut self.components
    }
}
