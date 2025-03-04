use crate::{
    components::DataComponentType,
    id,
    values::{Id, TextKinds, nbt::NbtCompound},
};

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
    pub const ITEM_NAME: DataComponentType<TextKinds> =
        DataComponentType::new(id![minecraft:item_name]);
    pub const LORE: DataComponentType<Vec<TextKinds>> = DataComponentType::new(id![minecraft:lore]);
}
