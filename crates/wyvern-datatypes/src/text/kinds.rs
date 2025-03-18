use voxidian_protocol::value::{
    Text as PtcText, TextColour, TextComponent, TextContent as PtcTextContent,
};

use super::TextMeta;

#[derive(Debug, Clone, PartialEq)]
pub struct Text {
    pub(crate) meta: TextMeta,
    pub(crate) content: TextContent,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TextContent {
    Group(Vec<Text>),
    Literal(String),
}

impl From<Text> for PtcText {
    fn from(value: Text) -> Self {
        match value.content {
            TextContent::Literal(text_literal) => {
                let mut text = PtcText::new();
                text.push(TextComponent {
                    content: PtcTextContent::Literal {
                        literal: text_literal,
                    },
                    style: value.meta.into(),
                    extra: vec![],
                });
                text
            }
            TextContent::Group(texts) => {
                let mut text = PtcText::new();
                for part in texts {
                    for compound in PtcText::from(part).into_components() {
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
