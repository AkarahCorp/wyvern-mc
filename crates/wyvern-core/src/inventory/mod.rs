mod data;
pub use data::*;

use crate::{actors::ActorResult, item::ItemStack};

pub trait Inventory {
    fn get_slot(&self, slot: usize) -> ActorResult<ItemStack>;
    fn set_slot(&mut self, slot: usize, item: ItemStack) -> ActorResult<()>;
}
