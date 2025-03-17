use core::f64;

use voxidian_protocol::{
    packet::s2c::play::{
        GameEvent, GameEventS2CPlayPacket, NumberFormat, ObjectiveKind, ObjectiveLocation,
        PlayerPositionS2CPlayPacket, SetBorderCenterS2CPlayPacket, SetBorderSizeS2CPlayPacket,
        SetBorderWarningDelayS2CPlayPacket, SetBorderWarningDistanceS2CPlayPacket,
        SetDisplayObjectiveS2CPlayPacket, SetHealthS2CPlayPacket, SetObjectiveS2CPlayPacket,
        SetScoreS2CPlayPacket, TeleportFlags, UpdateObjectiveAction,
    },
    value::{Text, VarInt},
};
use wyvern_components::{DataComponentHolder, DataComponentPatch};
use wyvern_datatypes::{gamemode::Gamemode, text::Texts};

use crate::{actors::ActorResult, entities::EntityComponents, item::ItemStack, runtime::Runtime};
use wyvern_values::{Vec3, id};

use super::{ConnectionData, Player, PlayerComponents};

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
                self.set(PlayerComponents::POSITION, location)?;
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

        if let Ok(velocity) = self.get(PlayerComponents::TELEPORT_VELOCITY) {
            if velocity != Vec3::new(f64::MIN, f64::MIN, f64::MIN) {
                self.set(
                    PlayerComponents::TELEPORT_SYNC_SENT,
                    self.get(PlayerComponents::TELEPORT_SYNC_SENT).unwrap_or(10) + 1,
                )?;
                self.set(
                    PlayerComponents::TELEPORT_VELOCITY,
                    Vec3::new(f64::MIN, f64::MIN, f64::MIN),
                )?;
                self.write_packet(PlayerPositionS2CPlayPacket {
                    teleport_id: VarInt::from(self.get(PlayerComponents::TELEPORT_SYNC_SENT)?),
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                    vx: velocity.x(),
                    vy: velocity.y(),
                    vz: velocity.z(),
                    adyaw_deg: 0.0,
                    adpitch_deg: 0.0,
                    flags: TeleportFlags {
                        relative_x: true,
                        relative_y: true,
                        relative_z: true,
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

        if let Ok(sidebar_present) = patch.added_fields().get(PlayerComponents::SIDEBAR_PRESENT) {
            if sidebar_present {
                self.write_packet(SetObjectiveS2CPlayPacket {
                    name: "wyvern_objective".into(),
                    action: UpdateObjectiveAction::Create {
                        value: Text::from(
                            self.get(PlayerComponents::SIDEBAR_NAME)
                                .unwrap_or_else(|_| Texts::literal("Untitled Objective").into()),
                        )
                        .to_nbt(),
                        kind: ObjectiveKind::Integer,
                        format: Some(NumberFormat::Blank),
                    },
                })?;
                self.write_packet(SetDisplayObjectiveS2CPlayPacket {
                    to: ObjectiveLocation::Sidebar,
                    name: "wyvern_objective".into(),
                })?;
            } else {
                self.write_packet(SetObjectiveS2CPlayPacket {
                    name: "wyvern_objective".into(),
                    action: UpdateObjectiveAction::Remove,
                })?;
            }
        }

        if self.get(PlayerComponents::SIDEBAR_PRESENT).unwrap_or(false) {
            if let Ok(sidebar_lines) = patch.added_fields().get(PlayerComponents::SIDEBAR_LINES) {
                for (idx, line) in sidebar_lines.into_iter().enumerate() {
                    let idx = usize::min(idx, i32::MAX as usize) as i32;
                    self.write_packet(SetScoreS2CPlayPacket {
                        entity_name: format!("line_{}", idx),
                        objective_name: "wyvern_objective".into(),
                        value: VarInt::new(i32::MAX - idx),
                        display_name: Some(Text::from(line).to_nbt()),
                        number_format: Some(NumberFormat::Blank),
                    })?;
                }
                self.write_packet(SetObjectiveS2CPlayPacket {
                    name: "wyvern_objective".into(),
                    action: UpdateObjectiveAction::Update {
                        value: Text::from(
                            self.get(PlayerComponents::SIDEBAR_NAME)
                                .unwrap_or_else(|_| Texts::literal("Untitled Objective").into()),
                        )
                        .to_nbt(),
                        kind: ObjectiveKind::Integer,
                        format: Some(NumberFormat::Blank),
                    },
                })?;
            }
        }

        if let Ok(health) = patch.added_fields().get(PlayerComponents::HEALTH) {
            self.write_packet(SetHealthS2CPlayPacket {
                hp: health.health,
                food: health.food.into(),
                sat: health.saturation,
            })?;
        }

        if let Ok(world_border) = patch.added_fields().get(PlayerComponents::WORLD_BORDER) {
            self.write_packet(SetBorderSizeS2CPlayPacket {
                diameter: world_border.size,
            })?;
            self.write_packet(SetBorderCenterS2CPlayPacket {
                x: world_border.center.x(),
                z: world_border.center.y(),
            })?;
            self.write_packet(SetBorderWarningDelayS2CPlayPacket {
                warning_time: world_border.warning_delay.into(),
            })?;
            self.write_packet(SetBorderWarningDistanceS2CPlayPacket {
                warning_dist: world_border.warning_distance.into(),
            })?;
        }

        self.set_saved_components(current_components.clone())?;

        Ok(())
    }
}

impl ConnectionData {
    pub fn update_self_entity(&mut self) -> ActorResult<()> {
        let dim = self.associated_data.dimension.clone().unwrap();
        let pos = self.get(PlayerComponents::POSITION)?;
        let dir = self.get(PlayerComponents::DIRECTION)?;
        let uuid = self.get(PlayerComponents::UUID)?;

        let main_hand_item = self
            .get_inv_slot(self.associated_data.held_slot as usize)
            .unwrap_or_else(|_| ItemStack::new(id![minecraft:air]));

        Runtime::spawn_task(move || {
            dim.get_entity(uuid).set(EntityComponents::POSITION, pos)?;
            dim.get_entity(uuid).set(EntityComponents::DIRECTION, dir)?;
            dim.get_entity(uuid)
                .set(EntityComponents::MAINHAND_ITEM, main_hand_item)?;
            Ok(())
        });

        Ok(())
    }
}
