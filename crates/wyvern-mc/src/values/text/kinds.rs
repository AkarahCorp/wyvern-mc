use voxidian_protocol::value::{TextColour, TextComponent};

use super::{Text, TextMeta};

#[derive(Debug, Clone)]
pub enum TextKinds {
    Literal(TextLiteral),
}

impl From<TextKinds> for TextComponent {
    fn from(value: TextKinds) -> Self {
        match value {
            TextKinds::Literal(text_literal) => text_literal.into(),
        }
    }
}
#[derive(Debug, Clone)]
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
