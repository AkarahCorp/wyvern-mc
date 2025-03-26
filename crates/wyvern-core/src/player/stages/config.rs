use voxidian_protocol::{
    packet::{
        Stage,
        c2s::config::{C2SConfigPackets, ResourcePackStatus},
        s2c::{
            config::{FinishConfigurationS2CConfigPacket, ResourcePackPushS2CConfigPacket},
            play::{Gamemode, LoginS2CPlayPacket, PlayerPositionS2CPlayPacket, TeleportFlags},
        },
    },
    registry::RegEntry,
    value::{PaintingVariant as PtcPaintingVariant, VarInt, WolfVariant as PtcWolfVariant},
};
use wyvern_values::Id;

use crate::{
    actors::ActorResult,
    player::ConnectionData,
    runtime::Runtime,
    server::{Server, registries::RegistryKeys},
};

impl ConnectionData {
    pub fn configuration_stage(&mut self) -> ActorResult<()> {
        self.read_packets(|packet: C2SConfigPackets, this: &mut Self| {
            log::debug!("Packet: {:?}", packet);
            {
                match packet {
                    C2SConfigPackets::CustomPayload(_packet) => {}
                    C2SConfigPackets::FinishConfiguration(_packet) => {
                        *this.stage.lock().unwrap() = Stage::Play;
                        this.associated_data.entity_id = this.connected_server.new_entity_id()?;
                        let id = this.associated_data.entity_id;
                        let p = this.as_actor();
                        Runtime::spawn_task(async move {
                            log::error!("GOT HERE AAAHHHH");
                            let default_dim = Server::get()?.default_dimension()?;
                            let default_dim = Server::get()?.dimension(default_dim)?;
                            p.write_packet(LoginS2CPlayPacket {
                                entity: id,
                                hardcore: false,
                                dims: Server::get()?
                                    .dimensions()?
                                    .iter()
                                    .map(|x| {
                                        x.name()
                                            .unwrap_or(Id::constant("minecraft", "overworld"))
                                            .into()
                                    })
                                    .collect::<Vec<_>>()
                                    .into(),
                                max_players: VarInt::from(100000),
                                view_dist: VarInt::from(10),
                                sim_dist: VarInt::from(10),
                                reduced_debug: false,
                                respawn_screen: true,
                                limited_crafting: false,
                                dim: unsafe {
                                    RegEntry::new_unchecked(
                                        Server::get()?
                                            .registries()?
                                            .get(RegistryKeys::DIMENSION_TYPE)
                                            .get_entry(default_dim.dimension_type()?)
                                            .unwrap_or(RegEntry::new_unchecked(u32::MAX))
                                            .id(),
                                    )
                                },
                                dim_name: default_dim.name()?.into(),
                                seed: 0,
                                gamemode: Gamemode::Survival,
                                old_gamemode: Gamemode::None,
                                is_debug: false,
                                is_flat: false,
                                death_loc: None,
                                portal_cooldown: VarInt::from(0),
                                sea_level: VarInt::from(64),
                                enforce_chat_reports: false,
                            })?;

                            p.write_packet(PlayerPositionS2CPlayPacket {
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
                            })?;
                            Ok(())
                        });
                    }
                    C2SConfigPackets::ResourcePack(packet) => match packet.status {
                        ResourcePackStatus::SuccessfullyDownloaded => {
                            this.write_packet(FinishConfigurationS2CConfigPacket);
                        }
                        ResourcePackStatus::Declined => todo!(),
                        ResourcePackStatus::FailedDownload => todo!(),
                        ResourcePackStatus::Accepted => {}
                        ResourcePackStatus::Downloaded => {}
                        ResourcePackStatus::InvalidURL => todo!(),
                        ResourcePackStatus::FailedReload => {}
                        ResourcePackStatus::Discarded => todo!(),
                    },
                    C2SConfigPackets::CookieResponse(_packet) => todo!(),
                    C2SConfigPackets::Pong(_packet) => todo!(),
                    C2SConfigPackets::ClientInformation(packet) => {
                        this.associated_data.render_distance = packet.info.view_distance as i32;
                    }
                    C2SConfigPackets::KeepAlive(_packet) => todo!(),
                    C2SConfigPackets::SelectKnownPacks(_packet) => {
                        this.write_packet(
                            this.connected_server
                                .registries()?
                                .get(RegistryKeys::BIOME)
                                .inner()
                                .to_registry_data_packet(),
                        );
                        this.write_packet(
                            this.connected_server
                                .registries()?
                                .get(RegistryKeys::DAMAGE_TYPE)
                                .inner()
                                .to_registry_data_packet(),
                        );
                        this.write_packet(
                            this.connected_server
                                .registries()?
                                .get(RegistryKeys::WOLF_VARIANT)
                                .map(|x| PtcWolfVariant::from(x.clone()))
                                .inner()
                                .to_registry_data_packet(),
                        );

                        this.write_packet(
                            this.connected_server
                                .registries()?
                                .get(RegistryKeys::PAINTING_VARIANT)
                                .map(|x| PtcPaintingVariant::from(x.clone()))
                                .inner()
                                .to_registry_data_packet(),
                        );

                        this.write_packet(
                            this.connected_server
                                .registries()?
                                .get(RegistryKeys::DIMENSION_TYPE)
                                .inner()
                                .to_registry_data_packet(),
                        );

                        this.write_packet(
                            this.connected_server
                                .registries()?
                                .get(RegistryKeys::CAT_VARIANT)
                                .inner()
                                .to_registry_data_packet(),
                        );

                        this.write_packet(
                            this.connected_server
                                .registries()?
                                .get(RegistryKeys::PIG_VARIANT)
                                .inner()
                                .to_registry_data_packet(),
                        );
                        this.write_packet(
                            this.connected_server
                                .registries()?
                                .get(RegistryKeys::COW_VARIANT)
                                .inner()
                                .to_registry_data_packet(),
                        );
                        this.write_packet(
                            this.connected_server
                                .registries()?
                                .get(RegistryKeys::CHICKEN_VARIANT)
                                .inner()
                                .to_registry_data_packet(),
                        );
                        this.write_packet(
                            this.connected_server
                                .registries()?
                                .get(RegistryKeys::FROG_VARIANT)
                                .inner()
                                .to_registry_data_packet(),
                        );
                        this.write_packet(
                            this.connected_server
                                .registries()?
                                .get(RegistryKeys::WOLF_SOUND_VARIANT)
                                .inner()
                                .to_registry_data_packet(),
                        );

                        if let Ok(pack) = Server::get()?.resource_pack() {
                            this.write_packet(ResourcePackPushS2CConfigPacket {
                                uuid: pack.uuid,
                                url: "http://localhost:62000".to_string(),
                                hash: "NoHash_".to_string(),
                                forced: true,
                                prompt: None,
                            });
                        } else {
                            this.write_packet(FinishConfigurationS2CConfigPacket);
                        }
                    }
                }
            }

            Ok(())
        })
    }
}
