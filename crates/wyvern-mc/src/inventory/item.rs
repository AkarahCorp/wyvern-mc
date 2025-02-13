use std::{collections::HashMap, sync::LazyLock};

use voxidian_protocol::{
    registry::Registry,
    value::{DataComponentTypes, DataComponents, Item, SlotData},
};

use crate::{components::ComponentHolder, values::Key};

use super::ItemComponents;

pub struct ItemType;

#[derive(Debug, Clone)]
pub struct ItemStack {
    id: Key<ItemType>,
    count: u16,
    pub(crate) components: HashMap<DataComponentTypes, DataComponents>,
}

impl ItemStack {
    pub fn new(id: Key<ItemType>) -> ItemStack {
        ItemStack {
            id,
            count: 1,
            components: HashMap::new(),
        }
    }

    pub fn air() -> ItemStack {
        ItemStack {
            id: Key::constant("minecraft", "air"),
            count: 0,
            components: HashMap::new(),
        }
    }
}

impl Default for ItemStack {
    fn default() -> Self {
        Self::air()
    }
}

pub(crate) static ITEM_REGISTRY: LazyLock<Registry<Item>> = LazyLock::new(Item::vanilla_registry);

impl From<ItemStack> for SlotData {
    fn from(value: ItemStack) -> Self {
        let components: Vec<DataComponents> = value.components.values().cloned().collect();
        let present_types: Vec<DataComponentTypes> =
            components.iter().map(|x| x.as_type()).collect();
        log::debug!("components: {:?}", components);
        SlotData {
            id: ITEM_REGISTRY.get_entry(&value.id.into()).unwrap(),
            count: (value.count as i32).into(),
            components,
            removed_components: DataComponentTypes::all_types()
                .into_iter()
                .filter(|x| !present_types.contains(x))
                .collect(),
        }
    }
}

impl ComponentHolder<ItemComponents> for ItemStack {}
