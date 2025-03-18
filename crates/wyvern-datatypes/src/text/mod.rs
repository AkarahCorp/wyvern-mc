use voxidian_protocol::value::{Text as PtcText, TextContent as PtcTextContent};

mod kinds;
pub use kinds::*;
mod meta;
pub use meta::*;

impl From<PtcText> for Text {
    fn from(value: PtcText) -> Self {
        let mut group = Vec::new();
        for component in value.into_components() {
            group.push(match component.content {
                PtcTextContent::Literal { literal } => Text::literal(literal).into(),
                PtcTextContent::Translate { .. } => todo!(),
                PtcTextContent::Keybind { .. } => todo!(),
            });
        }
        Text {
            meta: TextMeta::default(),
            content: TextContent::Group(group),
        }
    }
}

impl Text {
    pub fn literal(content: impl Into<String>) -> Text {
        Text {
            meta: TextMeta::default(),
            content: TextContent::Literal(content.into()),
        }
    }

    pub fn text_meta(&mut self) -> &mut TextMeta {
        &mut self.meta
    }

    pub fn with_color(mut self, color: TextColor) -> Self {
        self.text_meta().color = color;
        self
    }

    pub fn bold(mut self, bold: bool) -> Self {
        self.text_meta().style.bold = bold;
        self
    }

    pub fn italic(mut self, italic: bool) -> Self {
        self.text_meta().style.italic = italic;
        self
    }

    pub fn and_then(self, other: Text) -> Text {
        Text {
            meta: TextMeta::default(),
            content: TextContent::Group(vec![self, other]),
        }
    }
}
