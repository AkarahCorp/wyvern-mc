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
            TextKinds::Group(texts) => {
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

#[derive(Debug, Clone, PartialEq)]
pub struct TextGroup<L: Text, R: Text> {
    pub(crate) left: L,
    pub(crate) right: R,
    pub(crate) meta: TextMeta,
}

impl<L: Text, R: Text> From<TextGroup<L, R>> for TextKinds {
    fn from(value: TextGroup<L, R>) -> Self {
        TextKinds::Group(vec![value.left.into(), value.right.into()])
    }
}

impl<L: Text, R: Text> From<TextGroup<L, R>> for PtcText {
    fn from(value: TextGroup<L, R>) -> Self {
        let mut text = PtcText::new();
        let vec: Vec<TextKinds> = vec![value.left.into(), value.right.into()];
        for part in vec {
            for compound in PtcText::from(part).into_components() {
                text.push(compound);
            }
        }
        text
    }
}

impl<L: Text, R: Text> From<TextGroup<L, R>> for TextComponent {
    fn from(_value: TextGroup<L, R>) -> Self {
        unimplemented!()
    }
}

impl<L: Text, R: Text> Text for TextGroup<L, R> {
    fn text_meta(&mut self) -> &mut TextMeta {
        &mut self.meta
    }
}
