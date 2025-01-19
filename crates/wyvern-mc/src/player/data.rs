use voxidian_protocol::value::Uuid;

use crate::dimension::Dimension;

pub struct PlayerData {
    pub(crate) uuid: Uuid,
    pub(crate) username: String,
    pub(crate) dimension: Option<Dimension>,
}

impl Default for PlayerData {
    fn default() -> Self {
        Self {
            uuid: Default::default(),
            username: Default::default(),
            dimension: None,
        }
    }
}
