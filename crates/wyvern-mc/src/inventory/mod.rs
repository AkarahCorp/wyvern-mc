mod item;
pub use item::*;
mod data;
pub use data::*;

pub trait Inventory {
    fn get_slot(&self, slot: usize) -> impl Future<Output = Option<ItemStack>> + Send;
    fn set_slot(&mut self, slot: usize, item: ItemStack) -> impl Future<Output = ()> + Send;
}
