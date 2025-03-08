use voxidian_protocol::value::Uuid;

use crate::{
    components::{DataComponentHolder, DataComponentType},
    id,
    item::ItemStack,
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

    pub const VELOCITY: DataComponentType<Vec3<f64>> =
        DataComponentType::new(id![minecraft:velocity]);
    pub const PHYSICS_ENABLED: DataComponentType<bool> =
        DataComponentType::new(id![minecraft:physics]);
    pub const GRAVITY_ENABLED: DataComponentType<bool> =
        DataComponentType::new(id![minecraft:gravity]);
    pub const DRAG_ENABLED: DataComponentType<bool> = DataComponentType::new(id![minecraft:drag]);

    pub const MAINHAND_ITEM: DataComponentType<ItemStack> =
        DataComponentType::new(id![minecraft:equipment/mainhand]);
    pub const OFFHAND_ITEM: DataComponentType<ItemStack> =
        DataComponentType::new(id![minecraft:equipment/offhand]);
    pub const BODY_ITEM: DataComponentType<ItemStack> =
        DataComponentType::new(id![minecraft:equipment/body]);
    pub const HELMET_ITEM: DataComponentType<ItemStack> =
        DataComponentType::new(id![minecraft:equipment/helmet]);
    pub const CHESTPLATE_ITEM: DataComponentType<ItemStack> =
        DataComponentType::new(id![minecraft:equipment/chestplate]);
    pub const LEGGINGS_ITEM: DataComponentType<ItemStack> =
        DataComponentType::new(id![minecraft:equipment/leggings]);
    pub const BOOTS_ITEM: DataComponentType<ItemStack> =
        DataComponentType::new(id![minecraft:equipment/boots]);
}

impl DataComponentHolder for EntityData {
    fn component_map(&self) -> &crate::components::DataComponentMap {
        &self.components
    }

    fn component_map_mut(&mut self) -> &mut crate::components::DataComponentMap {
        &mut self.components
    }
}
