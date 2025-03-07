use voxidian_protocol::packet::s2c::play::{GameEvent, GameEventS2CPlayPacket, Gamemode};

use crate::{actors::ActorResult, components::DataComponentPatch};

use super::{Player, PlayerComponents};

impl Player {
    pub(crate) fn update_components(&mut self) -> ActorResult<()> {
        let last_components = self.get_saved_components()?;
        let current_components = self.get_current_components()?;
        let patch = DataComponentPatch::from_maps(&last_components, &current_components);

        if patch
            .added_fields()
            .contains_type(&PlayerComponents::GAMEMODE)
        {
            let mode = current_components.get(PlayerComponents::GAMEMODE)?;
            self.write_packet(GameEventS2CPlayPacket {
                event: GameEvent::ChangeGameMode,
                value: match mode {
                    Gamemode::None => 0.0,
                    Gamemode::Survival => 0.0,
                    Gamemode::Creative => 1.0,
                    Gamemode::Adventure => 2.0,
                    Gamemode::Spectator => 3.0,
                },
            })?;
        }

        if patch
            .added_fields()
            .contains_type(&PlayerComponents::ATTRIBUTES)
        {
            let container = current_components.get(PlayerComponents::ATTRIBUTES)?;
            self.write_packet(container.into_packet(self.entity_id()?))?;
        }

        self.set_saved_components(current_components)?;
        Ok(())
    }
}
