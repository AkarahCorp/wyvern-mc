use std::sync::LazyLock;

use voxidian_protocol::{
    registry::Registry,
    value::{DataComponentTypes, DataComponents, Item, SlotData},
};

use crate::values::Key;

pub struct ItemType;

#[derive(Debug, Clone)]
pub struct ItemStack {
    id: Key<ItemType>,
    count: u16,
    components: Vec<DataComponents>,
}

impl ItemStack {
    pub fn new(id: Key<ItemType>) -> ItemStack {
        ItemStack {
            id,
            count: 1,
            components: Vec::new(),
        }
    }

    pub fn air() -> ItemStack {
        ItemStack {
            id: Key::constant("minecraft", "air"),
            count: 0,
            components: Vec::new(),
        }
    }
}

impl Default for ItemStack {
    fn default() -> Self {
        Self::air()
    }
}

static ITEM_REGISTRY: LazyLock<Registry<Item>> = LazyLock::new(Item::vanilla_registry);

impl From<ItemStack> for SlotData {
    fn from(value: ItemStack) -> Self {
        SlotData {
            id: ITEM_REGISTRY.make_entry(&value.id.into()).unwrap(),
            count: (value.count as i32).into(),
            components: value.components,
            removed_components: DataComponentTypes::all_types(),
        }
    }
}
