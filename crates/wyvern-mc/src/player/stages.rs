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
                GameEvent, GameEventS2CPlayPacket, Gamemode, LoginS2CPlayPacket,
                PlayerPositionS2CPlayPacket, PongResponseS2CPlayPacket, TeleportFlags,
            },
            status::{
                PongResponseS2CStatusPacket, StatusResponse, StatusResponsePlayers,
                StatusResponseVersion,
            },
        },
    },
    registry::RegEntry,
    value::{Identifier, LengthPrefixHashMap, Text, VarInt},
};

use crate::{
    events::{PlayerCommandEvent, PlayerMoveEvent},
    values::Key,
};

use super::{ConnectionData, Player};

impl ConnectionData {
    pub async fn status_stage(&mut self) {
        self.read_packets(async |packet: C2SStatusPackets, this| match packet {
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
        })
        .await;
    }

    pub async fn login_stage(&mut self) {
        self.read_packets(
            async |packet: C2SLoginPackets, this: &mut Self| match packet {
                C2SLoginPackets::CustomQueryAnswer(_packet) => todo!(),
                C2SLoginPackets::LoginAcknowledged(_packet) => {
                    println!("login got acknowledged");
                    this.stage = Stage::Config;
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
            },
        )
        .await;
    }

    pub async fn configuration_stage(&mut self) {
        self.read_packets(async |packet: C2SConfigPackets, this: &mut Self| {
            match packet {
                C2SConfigPackets::CustomPayload(_packet) => {}
                C2SConfigPackets::FinishConfiguration(_packet) => {
                    this.stage = Stage::Play;
                    this.associated_data.entity_id = this.connected_server.get_entity_id().await;
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
                        gamemode: Gamemode::Creative,
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
                        y: 128.0,
                        z: 2.0,
                        vx: 0.0,
                        vy: 5.0,
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
                            .to_registry_data_packet(),
                    )
                    .await;
                    this.write_packet(
                        this.connected_server
                            .registries()
                            .await
                            .damage_types
                            .to_registry_data_packet(),
                    )
                    .await;
                    this.write_packet(
                        this.connected_server
                            .registries()
                            .await
                            .wolf_variants
                            .to_registry_data_packet(),
                    )
                    .await;
                    this.write_packet(
                        this.connected_server
                            .registries()
                            .await
                            .painting_variants
                            .to_registry_data_packet(),
                    )
                    .await;
                    this.write_packet(
                        this.connected_server
                            .registries()
                            .await
                            .dimension_types
                            .to_registry_data_packet(),
                    )
                    .await;
                    this.write_packet(FinishConfigurationS2CConfigPacket).await;
                }
            }
        })
        .await;
    }

    pub async fn play_phase(&mut self) {
        self.read_packets(async |packet: C2SPlayPackets, this: &mut Self| {
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
                        this.associated_data.dimension = this
                            .connected_server
                            .dimension(Key::new("wyvern", "root"))
                            .await;

                        this.write_packet(GameEventS2CPlayPacket {
                            event: GameEvent::WaitForChunks,
                            value: 0.0,
                        })
                        .await;

                        // this.write_packet(AddEntityS2CPlayPacket {
                        //     id: (this.associated_data.entity_id + 5).into(),
                        //     uuid: Uuid::new_v4(),
                        //     kind: this
                        //         .connected_server
                        //         .registries()
                        //         .await
                        //         .entity_types
                        //         .make_entry(&Identifier::new("minecraft", "villager"))
                        //         .unwrap(),
                        //     x: this.associated_data.last_position.x() + 1000.0,
                        //     y: this.associated_data.last_position.y() + 1000.0,
                        //     z: this.associated_data.last_position.z() + 1000.0,
                        //     pitch: Angle::of_deg(this.associated_data.last_direction.x()),
                        //     yaw: Angle::of_deg(this.associated_data.last_direction.y()),
                        //     head_yaw: Angle::of_deg(this.associated_data.last_direction.y()),
                        //     data: VarInt::from(0),
                        //     vel_x: 0,
                        //     vel_y: 0,
                        //     vel_z: 0,
                        // })
                        // .await;

                        for _conn in this.connected_server.connections().await.clone() {
                            let _data = this.associated_data.clone();
                            let _server = this.connected_server.clone();

                            // TODO: figure out player spawning logic
                            // tokio::spawn(async move {
                            //     conn.write_packet(BundleDelimiterS2CPlayPacket).await;

                            //     conn.write_packet(SetEntityDataS2CPlayPacket {
                            //         entity: (data.entity_id + 10).into(),
                            //         data: EntityMetadata::new()
                            //     }).await;
                            //     conn.write_packet(BundleDelimiterS2CPlayPacket).await;
                            // });
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

                    // this.write_packet(EntityPositionSyncS2CPlayPacket {
                    //     entity_id: (this.associated_data.entity_id + 5).into(),
                    //     x: this.associated_data.last_position.x(),
                    //     y: this.associated_data.last_position.y(),
                    //     z: this.associated_data.last_position.z(),
                    //     vx: 0.0,
                    //     vy: 0.0,
                    //     vz: 0.0,
                    //     yaw: 15.0,
                    //     pitch: 27.0,
                    //     on_ground: false,
                    // })
                    // .await;

                    this.connected_server.spawn_event(PlayerMoveEvent {
                        player: Player {
                            sender: this.sender.clone(),
                        },
                        new_position: this.associated_data.last_position.clone(),
                        new_direction: this.associated_data.last_direction.clone(),
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
                        new_position: this.associated_data.last_position.clone(),
                        new_direction: this.associated_data.last_direction.clone(),
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
                        new_position: this.associated_data.last_position.clone(),
                        new_direction: this.associated_data.last_direction.clone(),
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
                packet => {
                    println!("Received unknown play packet: {:?}", packet);
                }
            }
        })
        .await;
    }
}
