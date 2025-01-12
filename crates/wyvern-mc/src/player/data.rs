use voxidian_protocol::value::Uuid;

pub struct PlayerData {
    pub(crate) uuid: Uuid,
    pub(crate) username: String
}

impl Default for PlayerData {
    fn default() -> Self {
        Self { 
            uuid: Default::default(), 
            username: Default::default() 
        }
    }
}