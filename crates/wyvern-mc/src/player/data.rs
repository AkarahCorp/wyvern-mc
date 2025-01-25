use crate::{dimension::Dimension, values::Position};
use voxidian_protocol::{packet::c2s::play::InputFlags, value::Uuid};

pub struct PlayerData {
    pub(crate) uuid: Uuid,
    pub(crate) username: String,
    pub(crate) dimension: Option<Dimension>,

    #[allow(unused)]
    pub(crate) teleport_sync: i32,
    pub(crate) last_position: Position<f64, f64>,
    pub(crate) last_chunk_position: Position<i32>,
    pub(crate) loaded_chunks: Vec<Position<i32>>,
    pub(crate) render_distance: i32,

    pub(crate) input_flags: InputFlags,
}

impl Default for PlayerData {
    fn default() -> Self {
        Self {
            uuid: Default::default(),
            username: Default::default(),
            dimension: None,

            teleport_sync: 0,
            last_position: Position::new_angled(0.0, 0.0, 0.0, 0.0, 0.0),

            last_chunk_position: Position::new(0, 1, 0),
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
        }
    }
}
