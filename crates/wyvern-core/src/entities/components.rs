use voxidian_protocol::value::Uuid;
use wyvern_components::{DataComponentHolder, DataComponentMap, DataComponentType};

use crate::item::ItemStack;

use wyvern_values::{DVec3, Id, Vec2, id};

use super::EntityData;

pub struct EntityComponents;

impl EntityComponents {
    pub const ENTITY_TYPE: DataComponentType<Id> =
        DataComponentType::new(id![minecraft:entity_type]);
    pub const POSITION: DataComponentType<DVec3> = DataComponentType::new(id![minecraft:position]);
    pub const DIRECTION: DataComponentType<Vec2> = DataComponentType::new(id![minecraft:direction]);
    pub const UUID: DataComponentType<Uuid> = DataComponentType::new(id![minecraft:uuid]);
    pub const ENTITY_ID: DataComponentType<i32> = DataComponentType::new(id![minecraft:entity_id]);

    pub const PLAYER_CONTROLLED: DataComponentType<bool> =
        DataComponentType::new(id![minecraft:player_controlled]);
    pub const PLAYER_SKIN: DataComponentType<PlayerSkinData> =
        DataComponentType::new(id![minecraft:player_skin]);

    pub const VELOCITY: DataComponentType<DVec3> = DataComponentType::new(id![minecraft:velocity]);
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
    fn component_map(&self) -> &DataComponentMap {
        &self.components
    }

    fn component_map_mut(&mut self) -> &mut DataComponentMap {
        &mut self.components
    }
}

#[derive(Debug, Clone, Hash, PartialEq)]
pub struct PlayerSkinData {
    pub texture: String,
    pub signature: String,
}

impl PlayerSkinData {
    pub fn new(texture: &str, signature: &str) -> Self {
        PlayerSkinData {
            texture: texture.into(),
            signature: signature.into(),
        }
    }
}
