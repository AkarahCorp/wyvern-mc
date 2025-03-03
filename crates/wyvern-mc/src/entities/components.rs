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
        DataComponentType::new(0, id![minecraft:entity_id]);
    pub const POSITION: DataComponentType<Vec3<f64>> =
        DataComponentType::new(1, id![minecraft:position]);
    pub const DIRECTION: DataComponentType<Vec2<f32>> =
        DataComponentType::new(2, id![minecraft:direction]);
    pub const UUID: DataComponentType<Uuid> = DataComponentType::new(3, id![minecraft:uuid]);
    pub const ENTITY_ID: DataComponentType<i32> =
        DataComponentType::new(4, id![minecraft:entity_id]);
}

impl DataComponentHolder for EntityData {
    fn component_map(&self) -> &crate::components::DataComponentMap {
        &self.components
    }

    fn component_map_mut(&mut self) -> &mut crate::components::DataComponentMap {
        &mut self.components
    }
}
