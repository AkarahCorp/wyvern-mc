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
}

pub struct Texts;

impl Texts {
    pub fn literal(content: impl Into<String>) -> TextLiteral {
        TextLiteral {
            content: content.into(),
            meta: TextMeta {
                color: TextColor {
                    r: 255,
                    g: 255,
                    b: 255,
                },
                style: TextStyle {
                    bold: false,
                    italic: false,
                },
                children: Vec::new(),
            },
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
