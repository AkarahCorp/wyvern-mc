use voxidian_protocol::value::TextComponent;

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
