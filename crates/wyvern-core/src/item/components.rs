use voxidian_protocol::packet::s2c::play::EquipmentSlot as PtcEquipmentSlot;

use wyvern_components::DataComponentType;
use wyvern_datatypes::{nbt::NbtCompound, text::Text};
use wyvern_values::{Id, id};

pub struct ItemComponents;

impl ItemComponents {
    pub const ITEM_COUNT: DataComponentType<u16> =
        DataComponentType::new(id![minecraft:item_count]);
    pub const MAX_DAMAGE: DataComponentType<i32> =
        DataComponentType::new(id![minecraft:max_damage]);
    pub const DAMAGE: DataComponentType<i32> = DataComponentType::new(id![minecraft:damage]);
    pub const ITEM_MODEL: DataComponentType<Id> = DataComponentType::new(id![minecraft:item_model]);
    pub const CUSTOM_DATA: DataComponentType<NbtCompound> =
        DataComponentType::new(id![minecraft:custom_data]);
    pub const ITEM_NAME: DataComponentType<Text> = DataComponentType::new(id![minecraft:item_name]);
    pub const LORE: DataComponentType<Vec<Text>> = DataComponentType::new(id![minecraft:lore]);
    pub const EQUIPPABLE: DataComponentType<EquippableComponent> =
        DataComponentType::new(id![minecraft:equippable]);
}

#[derive(Debug, Clone, PartialEq)]
pub enum EquipmentSlot {
    Mainhand,
    Offhand,

    Helmet,
    Chestplate,
    Leggings,
    Boots,

    Body,
}

impl From<PtcEquipmentSlot> for EquipmentSlot {
    fn from(value: PtcEquipmentSlot) -> Self {
        match value {
            PtcEquipmentSlot::Mainhand => EquipmentSlot::Mainhand,
            PtcEquipmentSlot::Offhand => EquipmentSlot::Offhand,
            PtcEquipmentSlot::Boots => EquipmentSlot::Boots,
            PtcEquipmentSlot::Leggings => EquipmentSlot::Leggings,
            PtcEquipmentSlot::Chestplate => EquipmentSlot::Chestplate,
            PtcEquipmentSlot::Helmet => EquipmentSlot::Helmet,
            PtcEquipmentSlot::Body => EquipmentSlot::Body,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EquippableComponent {
    pub slot: EquipmentSlot,
    pub equip_sound: Id,
    pub model: Id,
}
