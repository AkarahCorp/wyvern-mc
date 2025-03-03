use crate::{
    components::DataComponentType,
    id,
    values::{Id, TextKinds, nbt::Nbt},
};

pub struct ItemComponents;

impl ItemComponents {
    pub const ITEM_COUNT: DataComponentType<u16> =
        DataComponentType::new(0, id![minecraft:item_count]);
    pub const MAX_DAMAGE: DataComponentType<i32> =
        DataComponentType::new(1, id![minecraft:max_damage]);
    pub const DAMAGE: DataComponentType<i32> = DataComponentType::new(2, id![minecraft:damage]);
    pub const ITEM_MODEL: DataComponentType<Id> =
        DataComponentType::new(3, id![minecraft:item_model]);
    pub const CUSTOM_DATA: DataComponentType<Nbt> =
        DataComponentType::new(4, id![minecraft:custom_data]);
    pub const ITEM_NAME: DataComponentType<TextKinds> =
        DataComponentType::new(5, id![minecraft:item_name]);
    pub const LORE: DataComponentType<Vec<TextKinds>> =
        DataComponentType::new(6, id![minecraft:lore]);
}
