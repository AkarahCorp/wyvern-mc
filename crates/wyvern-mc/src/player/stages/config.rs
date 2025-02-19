use voxidian_protocol::{
    packet::{
        Stage,
        c2s::config::C2SConfigPackets,
        s2c::{
            config::FinishConfigurationS2CConfigPacket,
            play::{Gamemode, LoginS2CPlayPacket, PlayerPositionS2CPlayPacket, TeleportFlags},
        },
    },
    registry::RegEntry,
    value::{Identifier, VarInt},
};

use crate::{actors::ActorResult, player::ConnectionData};

impl ConnectionData {
    pub async fn configuration_stage(&mut self) -> ActorResult<()> {
        self.read_packets(async |packet: C2SConfigPackets, this: &mut Self| {
            log::debug!("Packet: {:?}", packet);
            {
                match packet {
                    C2SConfigPackets::CustomPayload(_packet) => {}
                    C2SConfigPackets::FinishConfiguration(_packet) => {
                        *this.stage.lock().unwrap() = Stage::Play;
                        this.associated_data.entity_id =
                            this.connected_server.new_entity_id().await?;
                        this.write_packet(LoginS2CPlayPacket {
                            entity: this.associated_data.entity_id,
                            hardcore: false,
                            // fake dimensions so we can control client w/o extra storage
                            dims: vec![Identifier::new("wyvern", "fake")].into(),
                            max_players: VarInt::from(100),
                            view_dist: VarInt::from(16),
                            sim_dist: VarInt::from(16),
                            reduced_debug: false,
                            respawn_screen: true,
                            limited_crafting: false,
                            // TODO: Turn this into an actual Dimension Type lookup for
                            // the root dimension
                            dim: unsafe { RegEntry::new_unchecked(0) },
                            dim_name: Identifier::new("wyvern", "fake"),
                            seed: 0,
                            gamemode: Gamemode::Survival,
                            old_gamemode: Gamemode::None,
                            is_debug: false,
                            is_flat: false,
                            death_loc: None,
                            portal_cooldown: VarInt::from(0),
                            sea_level: VarInt::from(64),
                            enforce_chat_reports: false,
                        })
                        .await;

                        this.write_packet(PlayerPositionS2CPlayPacket {
                            teleport_id: VarInt::from(0),
                            x: 1.0,
                            y: 32.0,
                            z: 2.0,
                            vx: 0.0,
                            vy: 0.5,
                            vz: 0.0,
                            adyaw_deg: 0.0,
                            adpitch_deg: 0.0,
                            flags: TeleportFlags {
                                relative_x: false,
                                relative_y: false,
                                relative_z: false,
                                relative_pitch: false,
                                relative_yaw: false,
                                relative_vx: false,
                                relative_vy: false,
                                relative_vz: false,
                                rotate_velocity: false,
                            },
                        })
                        .await;
                    }
                    C2SConfigPackets::ResourcePack(_packet) => todo!(),
                    C2SConfigPackets::CookieResponse(_packet) => todo!(),
                    C2SConfigPackets::Pong(_packet) => todo!(),
                    C2SConfigPackets::ClientInformation(packet) => {
                        this.associated_data.render_distance = packet.info.view_distance as i32;
                    }
                    C2SConfigPackets::KeepAlive(_packet) => todo!(),
                    C2SConfigPackets::SelectKnownPacks(_packet) => {
                        this.write_packet(
                            this.connected_server
                                .registries()
                                .await?
                                .biomes
                                .inner
                                .to_registry_data_packet(),
                        )
                        .await;
                        this.write_packet(
                            this.connected_server
                                .registries()
                                .await?
                                .damage_types
                                .inner
                                .to_registry_data_packet(),
                        )
                        .await;
                        this.write_packet(
                            this.connected_server
                                .registries()
                                .await?
                                .wolf_variants
                                .inner
                                .to_registry_data_packet(),
                        )
                        .await;

                        this.write_packet(
                            this.connected_server
                                .registries()
                                .await?
                                .painting_variants
                                .inner
                                .to_registry_data_packet(),
                        )
                        .await;

                        this.write_packet(
                            this.connected_server
                                .registries()
                                .await?
                                .dimension_types
                                .inner
                                .to_registry_data_packet(),
                        )
                        .await;

                        this.write_packet(FinishConfigurationS2CConfigPacket).await;
                    }
                }
            }

            Ok(())
        })
        .await
    }
}
