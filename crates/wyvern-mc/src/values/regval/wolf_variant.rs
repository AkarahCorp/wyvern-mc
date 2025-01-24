use crate::values::{Key, resource::Texture};
use voxidian_protocol::value::{Biome, Identifier, WolfVariant as PtcWolfVariant};

pub struct WolfVariant {
    pub angry_texture: Key<Texture>,
    pub wild_texture: Key<Texture>,
    pub tame_texture: Key<Texture>,
    pub biomes: Vec<Key<Biome>>,
}

impl From<WolfVariant> for PtcWolfVariant {
    fn from(value: WolfVariant) -> Self {
        PtcWolfVariant {
            wild_texture: value.wild_texture.into(),
            tame_texture: value.tame_texture.into(),
            angry_texture: value.angry_texture.into(),
            biomes: value
                .biomes
                .into_iter()
                .map(|x| x.into())
                .collect::<Vec<Identifier>>(),
        }
    }
}
