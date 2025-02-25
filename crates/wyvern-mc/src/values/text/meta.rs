use super::TextKinds;

pub struct TextMeta {
    pub(crate) color: TextColor,
    pub(crate) style: TextStyle,
    #[allow(unused)] // currently uneditable by vxptc :(
    pub(crate) children: Vec<TextKinds>,
}

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

pub struct TextStyle {
    pub(crate) italic: bool,
    pub(crate) bold: bool,
}
