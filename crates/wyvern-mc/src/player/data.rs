use voxidian_protocol::value::Uuid;

use crate::{dimension::Dimension, values::position::Position};

pub struct PlayerData {
    pub(crate) uuid: Uuid,
    pub(crate) username: String,
    pub(crate) dimension: Option<Dimension>,

    #[allow(unused)]
    pub(crate) teleport_sync: i32,
    pub(crate) last_position: Position<f64, f64>,
    pub(crate) loaded_chunks: Vec<Position<i32>>,
    pub(crate) render_distance: i32,
}

impl Default for PlayerData {
    fn default() -> Self {
        Self {
            uuid: Default::default(),
            username: Default::default(),
            dimension: None,

            teleport_sync: 0,
            last_position: Position::new_angled(0.0, 0.0, 0.0, 0.0, 0.0),
            loaded_chunks: Vec::new(),
            render_distance: 2,
        }
    }
}
