mod components;
pub use components::*;
mod conversion;

use std::sync::LazyLock;

use voxidian_protocol::{registry::Registry, value::Item};

use crate::{
    components::{DataComponentHolder, DataComponentMap},
    values::Id,
};

pub struct ItemType;

#[derive(Clone, Debug, PartialEq)]
pub struct ItemStack {
    pub(crate) id: Id,
    pub(crate) map: DataComponentMap,
}

impl DataComponentHolder for ItemStack {
    fn component_map(&self) -> &crate::components::DataComponentMap {
        &self.map
    }

    fn component_map_mut(&mut self) -> &mut crate::components::DataComponentMap {
        &mut self.map
    }
}

impl ItemStack {
    pub fn new(id: Id) -> ItemStack {
        ItemStack {
            id: id.clone(),
            map: DataComponentMap::new()
                .with(ItemComponents::ITEM_COUNT, 1)
                .with(ItemComponents::ITEM_MODEL, id),
        }
    }

    pub fn air() -> ItemStack {
        ItemStack {
            id: Id::constant("minecraft", "air"),
            map: DataComponentMap::new()
                .with(ItemComponents::ITEM_COUNT, 1)
                .with(ItemComponents::ITEM_MODEL, Id::constant("minecraft", "air")),
        }
    }

    pub fn kind(&self) -> Id {
        self.id.clone()
    }
}

impl Default for ItemStack {
    fn default() -> Self {
        Self::air()
    }
}

pub(crate) static ITEM_REGISTRY: LazyLock<Registry<Item>> = LazyLock::new(Item::vanilla_registry);
