use voxidian_protocol::value::{Text as PtcText, TextColour, TextComponent};

use super::{Text, TextMeta};

#[derive(Debug, Clone, PartialEq)]
pub enum TextKinds {
    Group(Vec<TextKinds>),
    Literal(TextLiteral),
}

impl From<TextKinds> for PtcText {
    fn from(value: TextKinds) -> Self {
        match value {
            TextKinds::Literal(text_literal) => {
                let mut text = PtcText::new();
                text.push(text_literal.into());
                text
            }
            TextKinds::Group(items) => {
                let mut text = PtcText::new();
                for element in items {
                    let text2: PtcText = element.into();
                    for compound in text2.into_components() {
                        text.push(compound);
                    }
                }
                text
            }
        }
    }
}
#[derive(Debug, Clone, PartialEq)]
pub struct TextLiteral {
    pub(crate) content: String,
    pub(crate) meta: TextMeta,
}

impl From<TextLiteral> for TextComponent {
    fn from(value: TextLiteral) -> Self {
        TextComponent::of_literal(value.content)
            .bold(value.meta.style.bold)
            .italic(value.meta.style.italic)
            .colour(TextColour::RGB(
                value.meta.color.r,
                value.meta.color.g,
                value.meta.color.b,
            ))
    }
}

impl From<TextLiteral> for TextKinds {
    fn from(value: TextLiteral) -> Self {
        TextKinds::Literal(value)
    }
}

impl Text for TextLiteral {
    fn text_meta(&mut self) -> &mut TextMeta {
        &mut self.meta
    }
}
