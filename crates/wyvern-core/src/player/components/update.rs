use core::f64;

use voxidian_protocol::{
    packet::s2c::play::{
        GameEvent, GameEventS2CPlayPacket, NumberFormat, ObjectiveKind, ObjectiveLocation,
        PlayerPositionS2CPlayPacket, SetBorderCenterS2CPlayPacket, SetBorderSizeS2CPlayPacket,
        SetBorderWarningDelayS2CPlayPacket, SetBorderWarningDistanceS2CPlayPacket,
        SetDisplayObjectiveS2CPlayPacket, SetExperienceS2CPlayPacket, SetHealthS2CPlayPacket,
        SetObjectiveS2CPlayPacket, SetScoreS2CPlayPacket, TeleportFlags, UpdateObjectiveAction,
    },
    value::{Text as PtcText, VarInt},
};
use wyvern_components::{DataComponentHolder, DataComponentPatch};
use wyvern_datatypes::{gamemode::Gamemode, text::Text};

use crate::{
    actors::ActorResult, entities::EntityComponents, item::ItemStack, player::ConnectionData,
    runtime::Runtime,
};
use wyvern_values::{DVec3, id};

use super::{Player, PlayerComponents};

impl Player {
    pub(crate) fn update_components(&mut self) -> ActorResult<()> {
        let current_components = self.get_current_components()?;
        let last_components = self.get_saved_components()?;
        let patch = DataComponentPatch::from_maps(&last_components, &current_components);

        self.update_gamemode(&patch)?;
        self.update_sidebar(&patch)?;
        self.update_stats(&patch)?;
        self.update_teleport(&patch)?;
        self.update_velocity(&patch)?;
        self.update_attributes(&patch)?;

        self.set_saved_components(current_components.clone())?;

        Ok(())
    }

    pub(crate) fn update_gamemode(&mut self, patch: &DataComponentPatch) -> ActorResult<()> {
        if patch
            .added_fields()
            .contains_type(&PlayerComponents::GAMEMODE)
        {
            let mode = patch.added_fields().get(PlayerComponents::GAMEMODE)?;
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
        Ok(())
    }

    pub(crate) fn update_attributes(&mut self, patch: &DataComponentPatch) -> ActorResult<()> {
        if patch
            .added_fields()
            .contains_type(&PlayerComponents::ATTRIBUTES)
        {
            let container = patch.added_fields().get(PlayerComponents::ATTRIBUTES)?;
            self.write_packet(container.into_packet(self.entity_id()?))?;
        }
        Ok(())
    }

    pub(crate) fn update_teleport(&mut self, _patch: &DataComponentPatch) -> ActorResult<()> {
        if let Ok(location) = self.get(PlayerComponents::TELEPORT_POSITION) {
            if location != DVec3::new(f64::MIN, f64::MIN, f64::MIN) {
                self.set(
                    PlayerComponents::TELEPORT_SYNC_SENT,
                    self.get(PlayerComponents::TELEPORT_SYNC_SENT).unwrap_or(10) + 1,
                )?;
                self.set(
                    PlayerComponents::TELEPORT_POSITION,
                    DVec3::new(f64::MIN, f64::MIN, f64::MIN),
                )?;
                self.set(PlayerComponents::POSITION, location)?;
                self.write_packet(PlayerPositionS2CPlayPacket {
                    teleport_id: VarInt::from(self.get(PlayerComponents::TELEPORT_SYNC_SENT)?),
                    x: location[0],
                    y: location[1],
                    z: location[2],
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
        Ok(())
    }

    pub(crate) fn update_velocity(&mut self, _patch: &DataComponentPatch) -> ActorResult<()> {
        if let Ok(velocity) = self.get(PlayerComponents::TELEPORT_VELOCITY) {
            if velocity != DVec3::new(f64::MIN, f64::MIN, f64::MIN) {
                self.set(
                    PlayerComponents::TELEPORT_SYNC_SENT,
                    self.get(PlayerComponents::TELEPORT_SYNC_SENT).unwrap_or(10) + 1,
                )?;
                self.set(
                    PlayerComponents::TELEPORT_VELOCITY,
                    DVec3::new(f64::MIN, f64::MIN, f64::MIN),
                )?;
                self.write_packet(PlayerPositionS2CPlayPacket {
                    teleport_id: VarInt::from(self.get(PlayerComponents::TELEPORT_SYNC_SENT)?),
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                    vx: velocity[0],
                    vy: velocity[1],
                    vz: velocity[2],
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
        Ok(())
    }

    pub(crate) fn update_sidebar(&mut self, patch: &DataComponentPatch) -> ActorResult<()> {
        if let Ok(sidebar_present) = patch.added_fields().get(PlayerComponents::SIDEBAR_PRESENT) {
            if sidebar_present {
                self.write_packet(SetObjectiveS2CPlayPacket {
                    name: "wyvern_objective".into(),
                    action: UpdateObjectiveAction::Create {
                        value: PtcText::from(
                            self.get(PlayerComponents::SIDEBAR_NAME)
                                .unwrap_or_else(|_| Text::literal("Untitled Objective")),
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
                        display_name: Some(PtcText::from(line).to_nbt()),
                        number_format: Some(NumberFormat::Blank),
                    })?;
                }
                self.write_packet(SetObjectiveS2CPlayPacket {
                    name: "wyvern_objective".into(),
                    action: UpdateObjectiveAction::Update {
                        value: PtcText::from(
                            self.get(PlayerComponents::SIDEBAR_NAME)
                                .unwrap_or_else(|_| Text::literal("Untitled Objective")),
                        )
                        .to_nbt(),
                        kind: ObjectiveKind::Integer,
                        format: Some(NumberFormat::Blank),
                    },
                })?;
            }
        }
        Ok(())
    }

    pub(crate) fn update_stats(&mut self, patch: &DataComponentPatch) -> ActorResult<()> {
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
                x: world_border.center[0],
                z: world_border.center[1],
            })?;
            self.write_packet(SetBorderWarningDelayS2CPlayPacket {
                warning_time: world_border.warning_delay.into(),
            })?;
            self.write_packet(SetBorderWarningDistanceS2CPlayPacket {
                warning_dist: world_border.warning_distance.into(),
            })?;
        }

        if let Ok(experience) = patch.added_fields().get(PlayerComponents::EXPERIENCE) {
            self.write_packet(SetExperienceS2CPlayPacket {
                frac: experience.progress,
                level: experience.level.into(),
                total: (calculate_total_experience(experience.level, experience.progress) as i32)
                    .into(),
            })?;
        }
        Ok(())
    }
}

fn calculate_total_experience(level: i32, progress: f32) -> f32 {
    let progress = progress.clamp(0.0, 1.0);

    let experience_required = if level <= 15 {
        2.0 * level as f32 + 7.0
    } else if level <= 30 {
        5.0 * level as f32 - 38.0
    } else {
        9.0 * level as f32 - 158.0
    };

    let total_experience = if level <= 16 {
        (level as f32).powi(2) + 6.0 * level as f32
    } else if level <= 31 {
        2.5 * (level as f32).powi(2) - 40.5 * level as f32 + 360.0
    } else {
        4.5 * (level as f32).powi(2) - 162.5 * level as f32 + 2220.0
    };

    total_experience + progress * experience_required
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

        Runtime::spawn_task(async move {
            dim.get_entity(uuid).set(EntityComponents::POSITION, pos)?;
            dim.get_entity(uuid).set(EntityComponents::DIRECTION, dir)?;
            dim.get_entity(uuid)
                .set(EntityComponents::MAINHAND_ITEM, main_hand_item)?;
            Ok(())
        });

        Ok(())
    }
}
