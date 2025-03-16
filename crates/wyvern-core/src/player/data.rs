use std::time::Instant;

use crate::{dimension::Dimension, inventory::DataInventory, item::ItemStack};
use wyvern_datatypes::window::InventoryKind;
use wyvern_values::Vec2;

#[derive(Debug, Clone)]
pub struct PlayerData {
    pub(crate) dimension: Option<Dimension>,

    #[allow(unused)]
    pub(crate) last_chunk_position: Vec2<i32>,
    pub(crate) loaded_chunks: Vec<Vec2<i32>>,
    pub(crate) render_distance: i32,

    pub(crate) entity_id: i32,

    pub(crate) last_sent_keep_alive: Instant,

    pub(crate) inventory: DataInventory,
    pub(crate) screen: Option<(InventoryKind, DataInventory)>,
    pub(crate) window_id: i8,
    pub(crate) held_slot: i16,

    pub(crate) cursor_item: ItemStack,
}

impl Default for PlayerData {
    fn default() -> Self {
        Self {
            dimension: None,

            last_chunk_position: Vec2::new(0, 0),
            loaded_chunks: Vec::new(),

            render_distance: 2,
            entity_id: 0,
            last_sent_keep_alive: Instant::now(),

            inventory: DataInventory::new_filled(36, ItemStack::air),

            screen: None,
            held_slot: 36,

            cursor_item: ItemStack::air(),
            window_id: 0,
        }
    }
}
