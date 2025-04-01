pub mod values;
mod zip;

use std::collections::HashMap;

use wyvern_values::{Id, Uuid};

#[allow(unused)]
pub struct TexturePack {
    pub uuid: Uuid,

    pub textures: HashMap<Id, &'static [u8]>,
}

impl TexturePack {
    pub fn new() -> TexturePack {
        TexturePack {
            uuid: Uuid::new_v4(),
            textures: HashMap::new(),
        }
    }

    pub fn with_texture(mut self, name: Id, texture: &'static [u8]) -> Self {
        self.textures.insert(name, texture);
        self
    }
}

impl Default for TexturePack {
    fn default() -> Self {
        Self::new()
    }
}
