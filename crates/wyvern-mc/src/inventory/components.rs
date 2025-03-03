use crate::{components::DataComponentType, values::Id};

pub struct ItemComponents;

impl ItemComponents {
    pub const ITEM_COUNT: DataComponentType<u16> =
        DataComponentType::new(0, Id::constant("minecraft", "item_count"));
    pub const MAX_DAMAGE: DataComponentType<i32> =
        DataComponentType::new(1, Id::constant("minecraft", "max_damage"));
    pub const DAMAGE: DataComponentType<i32> =
        DataComponentType::new(2, Id::constant("minecraft", "damage"));
    pub const ITEM_MODEL: DataComponentType<Id> =
        DataComponentType::new(3, Id::constant("minecraft", "item_model"));
}
