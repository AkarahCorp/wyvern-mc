use voxidian_protocol::packet::s2c::play::ScreenWindowKind;

#[derive(Clone, Debug, PartialEq, Hash, Copy)]
pub enum InventoryKind {
    Chest1Row,
    Chest2Row,
    Chest3Row,
    Chest4Row,
    Chest5Row,
    Chest6Row,
}

impl From<InventoryKind> for ScreenWindowKind {
    fn from(value: InventoryKind) -> Self {
        match value {
            InventoryKind::Chest1Row => ScreenWindowKind::Generic9x1,
            InventoryKind::Chest2Row => ScreenWindowKind::Generic9x2,
            InventoryKind::Chest3Row => ScreenWindowKind::Generic9x3,
            InventoryKind::Chest4Row => ScreenWindowKind::Generic9x4,
            InventoryKind::Chest5Row => ScreenWindowKind::Generic9x5,
            InventoryKind::Chest6Row => ScreenWindowKind::Generic9x6,
        }
    }
}
