use crate::values::{Key, resource::Texture};
use datafix::serialization::{CodecAdapters, CodecOps, DefaultCodec, MapCodecBuilder};
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

impl<OT, O: CodecOps<OT>> DefaultCodec<OT, O> for WolfVariant {
    fn codec() -> impl datafix::serialization::Codec<Self, OT, O> {
        MapCodecBuilder::new()
            .field(Key::codec().field_of("angry", |w: &WolfVariant| &w.angry_texture))
            .field(Key::codec().field_of("wild", |w: &WolfVariant| &w.wild_texture))
            .field(Key::codec().field_of("tame", |w: &WolfVariant| &w.tame_texture))
            .field(
                Key::codec()
                    .list_of()
                    .field_of("biomes", |w: &WolfVariant| &w.biomes),
            )
            .build(
                |angry_texture, wild_texture, tame_texture, biomes| WolfVariant {
                    angry_texture,
                    wild_texture,
                    tame_texture,
                    biomes,
                },
            )
    }
}
