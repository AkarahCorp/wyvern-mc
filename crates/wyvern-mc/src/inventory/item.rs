use std::{
    collections::{HashMap, HashSet},
    sync::LazyLock,
};

use voxidian_protocol::{
    registry::Registry,
    value::{DataComponentTypes, DataComponents, Item, SlotData},
};

use crate::values::Id;

use super::Component;

pub struct ItemType;

#[derive(Debug, Clone)]
pub struct ItemStack {
    pub(crate) id: Id,
    pub(crate) count: u16,
    pub(crate) added_components: HashMap<DataComponentTypes, DataComponents>,
    pub(crate) removed_components: HashSet<DataComponentTypes>,
}

impl ItemStack {
    pub fn new(id: Id) -> ItemStack {
        ItemStack {
            id,
            count: 1,
            added_components: HashMap::new(),
            removed_components: HashSet::new(),
        }
    }

    pub fn air() -> ItemStack {
        ItemStack {
            id: Id::constant("minecraft", "air"),
            count: 0,
            added_components: HashMap::new(),
            removed_components: HashSet::new(),
        }
    }

    pub fn with<C: Component<ItemStack, V>, V>(mut self, component: C, value: V) -> Self {
        component.insert_component(&mut self, value);
        self
    }

    pub fn get<C: Component<ItemStack, V>, V>(&self, component: C) -> Option<V> {
        component.get_component(self)
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

impl From<ItemStack> for SlotData {
    fn from(value: ItemStack) -> Self {
        let components: Vec<DataComponents> = value.added_components.values().cloned().collect();
        let mut removed_components = Vec::new();
        for component in value.removed_components {
            removed_components.push(component);
        }
        SlotData {
            id: ITEM_REGISTRY.get_entry(&value.id.into()).unwrap(),
            count: (value.count as i32).into(),
            components,
            removed_components,
        }
    }
}

impl From<SlotData> for ItemStack {
    fn from(value: SlotData) -> Self {
        let mut added_components = HashMap::new();
        for component in value.components {
            added_components.insert(component.as_type(), component);
        }

        let mut removed_components = HashSet::new();
        for component in value.removed_components {
            removed_components.insert(component);
        }

        ItemStack {
            id: ITEM_REGISTRY.lookup(&value.id).unwrap().id.clone().into(),
            count: value.count.as_i32() as u16,
            added_components,
            removed_components,
        }
    }
}
