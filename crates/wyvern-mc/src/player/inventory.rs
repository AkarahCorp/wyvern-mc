use crate::{actors::ActorResult, inventory::Inventory};

use super::Player;

pub struct PlayerInventory {
    pub(crate) player: Player,
}

impl Inventory for PlayerInventory {
    async fn get_slot(&self, slot: usize) -> ActorResult<crate::inventory::ItemStack> {
        self.player.get_inv_slot(slot).await
    }

    async fn set_slot(
        &mut self,
        slot: usize,
        item: crate::inventory::ItemStack,
    ) -> ActorResult<()> {
        self.player.set_inv_slot(slot, item).await
    }
}
