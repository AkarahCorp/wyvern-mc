use std::time::Instant;

use crate::{
    dimension::Dimension,
    inventory::{DataInventory, ItemStack},
    values::{Vec2, Vec3},
};
use voxidian_protocol::{
    packet::{c2s::play::InputFlags, s2c::play::ScreenWindowKind},
    value::Uuid,
};

#[derive(Debug, Clone)]
pub struct PlayerData {
    pub(crate) uuid: Uuid,
    pub(crate) username: String,
    pub(crate) dimension: Option<Dimension>,

    #[allow(unused)]
    pub(crate) teleport_sync: i32,
    pub(crate) last_position: Vec3<f64>,
    pub(crate) last_direction: Vec2<f32>,
    pub(crate) last_chunk_position: Vec2<i32>,
    pub(crate) loaded_chunks: Vec<Vec2<i32>>,
    pub(crate) render_distance: i32,

    pub(crate) input_flags: InputFlags,
    pub(crate) is_loaded: bool,
    pub(crate) entity_id: i32,

    pub(crate) last_sent_keep_alive: Instant,

    pub(crate) inventory: DataInventory,
    pub(crate) screen: Option<ScreenWindowKind>,
    pub(crate) window_id: Option<i8>,
    pub(crate) held_slot: i16,
}

impl Default for PlayerData {
    fn default() -> Self {
        Self {
            uuid: Default::default(),
            username: Default::default(),
            dimension: None,

            teleport_sync: 0,
            last_position: Vec3::new(0.0, 0.0, 0.0),
            last_direction: Vec2::new(0.0, 0.0),

            last_chunk_position: Vec2::new(0, 0),
            loaded_chunks: Vec::new(),

            render_distance: 2,
            input_flags: InputFlags {
                forward: false,
                backward: false,
                left: false,
                right: false,
                sneak: false,
                sprint: false,
            },
            is_loaded: false,
            entity_id: 0,
            last_sent_keep_alive: Instant::now(),

            inventory: DataInventory::new_filled(36, ItemStack::air),
            screen: None,
            window_id: None,
            held_slot: 36,
        }
    }
}
