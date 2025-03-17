use voxidian_protocol::value::{Text as PtcText, TextComponent, TextContent};

mod kinds;
pub use kinds::*;
mod meta;
pub use meta::*;

pub trait Text: Into<TextComponent> + Into<TextKinds> {
    fn text_meta(&mut self) -> &mut TextMeta;

    fn with_color(mut self, color: TextColor) -> Self {
        self.text_meta().color = color;
        self
    }

    fn bold(mut self, bold: bool) -> Self {
        self.text_meta().style.bold = bold;
        self
    }

    fn italic(mut self, italic: bool) -> Self {
        self.text_meta().style.italic = italic;
        self
    }

    fn and_then(self, other: impl Text) -> impl Text {
        TextGroup {
            left: self,
            right: other,
            meta: TextMeta::default(),
        }
    }
}

pub struct Texts;

impl Texts {
    pub fn literal(content: impl Into<String>) -> TextLiteral {
        TextLiteral {
            content: content.into(),
            meta: TextMeta::default(),
        }
    }
}

impl From<PtcText> for TextKinds {
    fn from(value: PtcText) -> Self {
        let mut group = Vec::new();
        for component in value.into_components() {
            group.push(match component.content {
                TextContent::Literal { literal } => Texts::literal(literal).into(),
                TextContent::Translate { .. } => todo!(),
                TextContent::Keybind { .. } => todo!(),
            });
        }
        TextKinds::Group(group)
    }
}
