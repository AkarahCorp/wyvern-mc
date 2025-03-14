use crate::Id;
use voxidian_protocol::value::{PaintingVariant as PtcPaintingVariant, TextComponent};

pub struct PaintingVariant {
    pub asset: Id,
    pub width: u32,
    pub height: u32,
}

impl From<PaintingVariant> for PtcPaintingVariant {
    fn from(value: PaintingVariant) -> Self {
        PtcPaintingVariant {
            asset_id: value.asset.into(),
            height: value.height,
            width: value.width,
            author: TextComponent::of_literal("Author"),
            title: TextComponent::of_literal("Title"),
        }
    }
}
