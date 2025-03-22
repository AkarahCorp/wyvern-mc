use datafix::serialization::{CodecAdapters, CodecOps, DefaultCodec, MapCodecBuilder};
use voxidian_protocol::value::WolfVariant as PtcWolfVariant;
use wyvern_values::Id;

#[derive(Debug, Clone)]
pub struct WolfVariant {
    pub angry_texture: Id,
    pub wild_texture: Id,
    pub tame_texture: Id,
}

impl From<WolfVariant> for PtcWolfVariant {
    fn from(value: WolfVariant) -> Self {
        PtcWolfVariant {
            wild_texture: value.wild_texture.into(),
            tame_texture: value.tame_texture.into(),
            angry_texture: value.angry_texture.into(),
        }
    }
}

impl<OT: Clone, O: CodecOps<OT>> DefaultCodec<OT, O> for WolfVariant {
    fn codec() -> impl datafix::serialization::Codec<Self, OT, O> {
        MapCodecBuilder::new()
            .field(Id::codec().field_of("angry", |w: &WolfVariant| &w.angry_texture))
            .field(Id::codec().field_of("wild", |w: &WolfVariant| &w.wild_texture))
            .field(Id::codec().field_of("tame", |w: &WolfVariant| &w.tame_texture))
            .build(|angry_texture, wild_texture, tame_texture| WolfVariant {
                angry_texture,
                wild_texture,
                tame_texture,
            })
    }
}
