mod zip;

use wyvern_values::Uuid;

#[allow(unused)]
pub struct TexturePack {
    pub uuid: Uuid,
}

impl TexturePack {
    pub fn new() -> TexturePack {
        TexturePack {
            uuid: Uuid::new_v4(),
        }
    }
}

impl Default for TexturePack {
    fn default() -> Self {
        Self::new()
    }
}
