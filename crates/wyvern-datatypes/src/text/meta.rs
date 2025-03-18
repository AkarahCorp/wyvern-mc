use voxidian_protocol::value::{TextColour as PtcTextColor, TextStyle as PtcTextStyle};

use super::Text;

#[derive(Debug, Clone, PartialEq)]
pub struct TextMeta {
    pub(crate) color: TextColor,
    pub(crate) style: TextStyle,
    #[allow(unused)] // currently uneditable by vxptc :(
    pub(crate) children: Vec<Text>,
}

impl Default for TextMeta {
    fn default() -> Self {
        TextMeta {
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
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TextColor {
    pub(crate) r: u8,
    pub(crate) g: u8,
    pub(crate) b: u8,
}

impl TextColor {
    pub fn new(r: u8, g: u8, b: u8) -> TextColor {
        TextColor { r, g, b }
    }
}
#[derive(Debug, Clone, PartialEq)]
pub struct TextStyle {
    pub(crate) italic: bool,
    pub(crate) bold: bool,
}

impl From<TextMeta> for PtcTextStyle {
    fn from(value: TextMeta) -> Self {
        PtcTextStyle {
            colour: Some(PtcTextColor::RGB(
                value.color.r,
                value.color.g,
                value.color.b,
            )),
            font: None,
            bold: Some(value.style.bold),
            italic: Some(value.style.italic),
            underline: Some(value.style.italic),
            strikethrough: None,
            obfuscate: None,
            insertion: None,
            click_event: None,
            hover_event: None,
        }
    }
}
