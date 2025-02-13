use voxidian_protocol::{
    packet::{
        Stage,
        c2s::{
            config::C2SConfigPackets,
            login::C2SLoginPackets,
            play::{C2SPlayPackets, PlayerStatus},
            status::C2SStatusPackets,
        },
        s2c::{
            config::{
                FinishConfigurationS2CConfigPacket, KnownPack, SelectKnownPacksS2CConfigPacket,
            },
            login::LoginFinishedS2CLoginPacket,
            play::{
                AddEntityS2CPlayPacket, GameEvent, GameEventS2CPlayPacket, Gamemode,
                LoginS2CPlayPacket, PlayerPositionS2CPlayPacket, PongResponseS2CPlayPacket,
                TeleportFlags,
            },
            status::{
                PongResponseS2CStatusPacket, StatusResponse, StatusResponsePlayers,
                StatusResponseVersion,
            },
        },
    },
    registry::RegEntry,
    value::{Angle, Identifier, LengthPrefixHashMap, Text, VarInt},
};

use crate::{
    events::{PlayerCommandEvent, PlayerMoveEvent},
    inventory::{ITEM_REGISTRY, Inventory, ItemStack},
    values::Key,
};

use super::{ConnectionData, Player};

impl ConnectionData {
    pub async fn status_stage(&mut self) {
        self.read_packets(async |packet: C2SStatusPackets, this| {
            log::debug!("Packet: {:?}", packet);
            match packet {
                C2SStatusPackets::StatusRequest(_packet) => {
                    this.write_packet(
                        StatusResponse {
                            version: StatusResponseVersion {
                                name: "1.21.4".to_string(),
                                protocol: 769,
                            },
                            players: Some(StatusResponsePlayers {
                                online: 0,
                                max: 100,
                                sample: vec![],
                            }),
                            desc: Text::new(),
                            favicon_png_b64: "".to_string(),
                            enforce_chat_reports: false,
                            prevent_chat_reports: true,
                        }
                        .to_packet(),
                    )
                    .await;
                }
                C2SStatusPackets::PingRequest(packet) => {
                    this.write_packet(PongResponseS2CStatusPacket {
                        timestamp: packet.timestamp,
                    })
                    .await;
                }
            }
        })
        .await;
    }

    pub async fn login_stage(&mut self) {
        self.read_packets(async |packet: C2SLoginPackets, this: &mut Self| {
            log::debug!("Packet: {:?}", packet);
            match packet {
                C2SLoginPackets::CustomQueryAnswer(_packet) => todo!(),
                C2SLoginPackets::LoginAcknowledged(_packet) => {
                    *this.stage.lock().unwrap() = Stage::Config;
                    this.write_packet(SelectKnownPacksS2CConfigPacket {
                        known_packs: vec![KnownPack {
                            namespace: "minecraft".to_string(),
                            id: "core".to_string(),
                            version: "1.21.4".to_string(),
                        }]
                        .into(),
                    })
                    .await;
                }
                C2SLoginPackets::Key(_packet) => todo!(),
                C2SLoginPackets::Hello(packet) => {
                    this.associated_data.username = packet.username.clone();
                    this.associated_data.uuid = packet.uuid;
                    this.write_packet(LoginFinishedS2CLoginPacket {
                        uuid: packet.uuid,
                        username: packet.username,
                        props: LengthPrefixHashMap::new(),
                    })
                    .await;
                }
                C2SLoginPackets::CookieResponse(_packet) => todo!(),
            }
        })
        .await;
    }

    pub async fn configuration_stage(&mut self) {
        self.read_packets(async |packet: C2SConfigPackets, this: &mut Self| {
            log::debug!("Packet: {:?}", packet);
            {
                match packet {
                    C2SConfigPackets::CustomPayload(_packet) => {}
                    C2SConfigPackets::FinishConfiguration(_packet) => {
                        *this.stage.lock().unwrap() = Stage::Play;
                        this.associated_data.entity_id =
                            this.connected_server.get_entity_id().await;
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
                                .await
                                .biomes
                                .inner
                                .to_registry_data_packet(),
                        )
                        .await;
                        this.write_packet(
                            this.connected_server
                                .registries()
                                .await
                                .damage_types
                                .inner
                                .to_registry_data_packet(),
                        )
                        .await;
                        this.write_packet(
                            this.connected_server
                                .registries()
                                .await
                                .wolf_variants
                                .inner
                                .to_registry_data_packet(),
                        )
                        .await;

                        this.write_packet(
                            this.connected_server
                                .registries()
                                .await
                                .painting_variants
                                .inner
                                .to_registry_data_packet(),
                        )
                        .await;

                        this.write_packet(
                            this.connected_server
                                .registries()
                                .await
                                .dimension_types
                                .inner
                                .to_registry_data_packet(),
                        )
                        .await;

                        this.write_packet(FinishConfigurationS2CConfigPacket).await;
                    }
                }
            }
        })
        .await;
    }

    pub async fn play_phase(&mut self) {
        self.read_packets(async |packet: C2SPlayPackets, this: &mut Self| {
            log::debug!(
                "Player {:?} has sent packet: {:?}",
                this.associated_data.username,
                packet
            );
            match packet {
                C2SPlayPackets::PlayerLoaded(_packet) => {
                    this.associated_data.is_loaded = true;
                }
                C2SPlayPackets::ChatCommand(packet) => {
                    this.connected_server.spawn_event(PlayerCommandEvent {
                        player: Player {
                            sender: this.sender.clone(),
                        },
                        command: packet.command,
                    });
                }
                C2SPlayPackets::PlayerAction(packet) => match packet.status {
                    PlayerStatus::StartedDigging => {}
                    PlayerStatus::CancelledDigging => {}
                    PlayerStatus::FinishedDigging => {}
                    PlayerStatus::DropItemStack => {}
                    PlayerStatus::DropItem => {}
                    PlayerStatus::FinishUsingItem => {}
                    PlayerStatus::SwapItems => {}
                },
                C2SPlayPackets::AcceptTeleportation(packet) => {
                    if packet.teleport_id.as_i32() == 0 {
                        log::debug!("Setting dimension...");
                        this.associated_data.dimension = this
                            .connected_server
                            .dimension(Key::new("wyvern", "root"))
                            .await;

                        log::debug!("Sending game events chunk packet...");
                        this.write_packet(GameEventS2CPlayPacket {
                            event: GameEvent::WaitForChunks,
                            value: 0.0,
                        })
                        .await;

                        log::debug!("Sending all entities...");
                        for entity in this
                            .associated_data
                            .dimension
                            .as_mut()
                            .unwrap()
                            .get_all_entities()
                            .await
                        {
                            let position = entity.position().await;

                            log::debug!("Entity @ {:?}...", position);
                            this.write_packet(AddEntityS2CPlayPacket {
                                id: entity.entity_id().await.into(),
                                uuid: *entity.uuid(),
                                kind: this
                                    .connected_server
                                    .registries()
                                    .await
                                    .entity_types
                                    .get_entry(entity.entity_type().await.retype())
                                    .unwrap(),
                                x: position.0.x(),
                                y: position.0.x(),
                                z: position.0.x(),
                                pitch: Angle::of_deg(position.1.x()),
                                yaw: Angle::of_deg(position.1.y()),
                                head_yaw: Angle::of_deg(position.1.y()),
                                data: VarInt::from(0),
                                vel_x: 0,
                                vel_y: 0,
                                vel_z: 0,
                            })
                            .await;
                        }
                    }

                    this.send_chunks().await;
                }
                C2SPlayPackets::MovePlayerPos(packet) => {
                    this.associated_data.last_position = this
                        .associated_data
                        .last_position
                        .with_x(packet.x)
                        .with_y(packet.y)
                        .with_z(packet.z);

                    this.send_chunks().await;

                    this.connected_server.spawn_event(PlayerMoveEvent {
                        player: Player {
                            sender: this.sender.clone(),
                        },
                        new_position: this.associated_data.last_position,
                        new_direction: this.associated_data.last_direction,
                    });
                }
                C2SPlayPackets::MovePlayerPosRot(packet) => {
                    this.associated_data.last_position = this
                        .associated_data
                        .last_position
                        .with_x(packet.x)
                        .with_y(packet.y)
                        .with_z(packet.z);

                    this.associated_data.last_direction = this
                        .associated_data
                        .last_direction
                        .with_x(packet.pitch)
                        .with_y(packet.yaw);

                    this.send_chunks().await;

                    this.connected_server.spawn_event(PlayerMoveEvent {
                        player: Player {
                            sender: this.sender.clone(),
                        },
                        new_position: this.associated_data.last_position,
                        new_direction: this.associated_data.last_direction,
                    });
                }
                C2SPlayPackets::MovePlayerRot(packet) => {
                    this.associated_data.last_direction = this
                        .associated_data
                        .last_direction
                        .with_x(packet.pitch)
                        .with_y(packet.yaw);

                    this.connected_server.spawn_event(PlayerMoveEvent {
                        player: Player {
                            sender: this.sender.clone(),
                        },
                        new_position: this.associated_data.last_position,
                        new_direction: this.associated_data.last_direction,
                    });
                }
                C2SPlayPackets::ClientInformation(packet) => {
                    this.associated_data.render_distance = packet.info.view_distance as i32;
                }
                C2SPlayPackets::PlayerInput(packet) => {
                    this.associated_data.input_flags = packet.flags;
                }
                C2SPlayPackets::ClientTickEnd(_) => {}
                C2SPlayPackets::PingRequest(packet) => {
                    this.write_packet(PongResponseS2CPlayPacket(packet.id as u64))
                        .await;
                }
                C2SPlayPackets::ChunkBatchReceived(_packet) => {}

                C2SPlayPackets::SetCreativeModeSlot(packet) => {
                    let item = ITEM_REGISTRY.lookup(&packet.new_item.id).unwrap();
                    let stack = ItemStack::new(item.id.clone().into());

                    this.associated_data
                        .inventory
                        .set_slot(packet.slot as usize, stack)
                        .await;
                }
                packet => {
                    log::warn!(
                        "Received unknown play packet, this packet will be ignored. {:?}",
                        packet
                    );
                }
            }
        })
        .await;
    }
}
