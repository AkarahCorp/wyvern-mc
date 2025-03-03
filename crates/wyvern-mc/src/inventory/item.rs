use std::sync::LazyLock;

use voxidian_protocol::{
    registry::Registry,
    value::{Damage, DataComponents, Item, ItemModel, MaxDamage, SlotData, VarInt},
};

use crate::{
    components::{DataComponentHolder, DataComponentMap},
    values::Id,
};

use super::ItemComponents;

pub struct ItemType;

#[derive(Clone, Debug)]
pub struct ItemStack {
    pub(crate) id: Id,
    pub(crate) count: u16,
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
            id,
            count: 1,
            map: DataComponentMap::new(),
        }
    }

    pub fn air() -> ItemStack {
        ItemStack {
            id: Id::constant("minecraft", "air"),
            count: 0,
            map: DataComponentMap::new(),
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

impl From<ItemStack> for SlotData {
    fn from(value: ItemStack) -> Self {
        let mut components: Vec<DataComponents> = Vec::new();

        if let Ok(c) = value.get(ItemComponents::DAMAGE) {
            components.push(DataComponents::Damage(Damage {
                damage: VarInt::new(c),
            }));
        }
        if let Ok(amount) = value.get(ItemComponents::MAX_DAMAGE) {
            components.push(DataComponents::MaxDamage(MaxDamage {
                amount: VarInt::new(amount),
            }));
        }
        if let Ok(asset) = value.get(ItemComponents::ITEM_MODEL) {
            components.push(DataComponents::ItemModel(ItemModel {
                asset: asset.into(),
            }));
        }

        SlotData {
            id: ITEM_REGISTRY.get_entry(&value.id.into()).unwrap(),
            count: (value.count as i32).into(),
            components,
            removed_components: Vec::new(),
        }
    }
}

impl From<SlotData> for ItemStack {
    fn from(value: SlotData) -> Self {
        ItemStack {
            id: ITEM_REGISTRY.lookup(&value.id).unwrap().id.clone().into(),
            count: value.count.as_i32() as u16,
            map: DataComponentMap::new(),
        }
    }
}
