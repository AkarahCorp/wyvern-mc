mod item;
pub use item::*;
mod data;
pub use data::*;
mod components;
pub use components::*;

use crate::actors::ActorResult;

pub trait Inventory {
    fn get_slot(&self, slot: usize) -> impl Future<Output = ActorResult<ItemStack>> + Send;
    fn set_slot(
        &mut self,
        slot: usize,
        item: ItemStack,
    ) -> impl Future<Output = ActorResult<()>> + Send;
}
