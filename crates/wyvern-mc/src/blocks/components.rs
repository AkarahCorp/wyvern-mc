use crate::{components::DataComponentType, id, values::nbt::Nbt};

pub struct BlockComponents;

impl BlockComponents {
    pub const CUSTOM_DATA: DataComponentType<Nbt> =
        DataComponentType::new(id![minecraft:custom_data]);
    pub const SNOWY: DataComponentType<bool> = DataComponentType::new(id![minecraft:snowy]);
}
