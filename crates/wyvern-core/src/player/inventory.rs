use crate::{actors::ActorResult, inventory::Inventory};

use super::Player;

pub struct PlayerInventory {
    pub(crate) player: Player,
}

impl Inventory for PlayerInventory {
    fn get_slot(&self, slot: usize) -> ActorResult<crate::item::ItemStack> {
        self.player.get_inv_slot(slot)
    }

    fn set_slot(&mut self, slot: usize, item: crate::item::ItemStack) -> ActorResult<()> {
        self.player.set_inv_slot(slot, item)
    }
}
