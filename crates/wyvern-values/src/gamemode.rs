use voxidian_protocol::packet::s2c::play::Gamemode as PtcGamemode;

#[derive(Debug, Clone, Hash, PartialEq)]
pub enum Gamemode {
    Survival,
    Creative,
    Adventure,
    Spectator,
}

impl From<Gamemode> for PtcGamemode {
    fn from(value: Gamemode) -> Self {
        match value {
            Gamemode::Survival => PtcGamemode::Survival,
            Gamemode::Creative => PtcGamemode::Creative,
            Gamemode::Adventure => PtcGamemode::Adventure,
            Gamemode::Spectator => PtcGamemode::Spectator,
        }
    }
}
