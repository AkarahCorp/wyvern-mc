use std::{collections::HashMap, fmt::Debug};

use super::{Inventory, ItemStack};

#[derive(Clone)]
pub struct DataInventory {
    slots: HashMap<usize, ItemStack>,
}

impl Debug for DataInventory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DataInventory")
            .field("slots", &"...")
            .finish()
    }
}

impl DataInventory {
    pub fn new() -> DataInventory {
        DataInventory {
            slots: HashMap::new(),
        }
    }

    pub fn new_filled(slots: usize, f: fn() -> ItemStack) -> DataInventory {
        let mut map = HashMap::new();
        for idx in 0..slots {
            map.insert(idx, f());
        }
        DataInventory { slots: map }
    }
}

impl Default for DataInventory {
    fn default() -> Self {
        Self::new()
    }
}

impl Inventory for DataInventory {
    async fn get_slot(&self, slot: usize) -> Option<ItemStack> {
        self.slots.get(&slot).cloned()
    }

    async fn set_slot(&mut self, slot: usize, item: ItemStack) {
        self.slots.insert(slot, item);
    }
}
