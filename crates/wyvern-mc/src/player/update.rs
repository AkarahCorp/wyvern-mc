use core::f64;

use voxidian_protocol::{
    packet::s2c::play::{
        GameEvent, GameEventS2CPlayPacket, Gamemode, PlayerPositionS2CPlayPacket, TeleportFlags,
    },
    value::VarInt,
};

use crate::{actors::ActorResult, components::DataComponentPatch, values::Vec3};

use super::{Player, PlayerComponents};

impl Player {
    pub(crate) fn update_components(&mut self) -> ActorResult<()> {
        let current_components = self.get_current_components()?;
        let last_components = self.get_saved_components()?;
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

        if let Ok(location) = self.get(PlayerComponents::TELEPORT_POSITION) {
            if location != Vec3::new(f64::MIN, f64::MIN, f64::MIN) {
                self.set(
                    PlayerComponents::TELEPORT_SYNC_SENT,
                    self.get(PlayerComponents::TELEPORT_SYNC_SENT).unwrap_or(10) + 1,
                )?;
                self.set(
                    PlayerComponents::TELEPORT_POSITION,
                    Vec3::new(f64::MIN, f64::MIN, f64::MIN),
                )?;
                self.write_packet(PlayerPositionS2CPlayPacket {
                    teleport_id: VarInt::from(self.get(PlayerComponents::TELEPORT_SYNC_SENT)?),
                    x: location.x(),
                    y: location.y(),
                    z: location.z(),
                    vx: 0.0,
                    vy: 0.0,
                    vz: 0.0,
                    adyaw_deg: 0.0,
                    adpitch_deg: 0.0,
                    flags: TeleportFlags {
                        relative_x: false,
                        relative_y: false,
                        relative_z: false,
                        relative_pitch: true,
                        relative_yaw: true,
                        relative_vx: true,
                        relative_vy: true,
                        relative_vz: true,
                        rotate_velocity: false,
                    },
                })?;
            }
        }

        self.set_saved_components(current_components.clone())?;

        Ok(())
    }
}
