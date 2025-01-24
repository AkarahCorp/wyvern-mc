use std::fmt::Debug;

use voxidian_protocol::{
    packet::{
        PacketBuf, PacketEncode, PrefixedPacketEncode, Stage,
        c2s::{
            config::C2SConfigPackets, login::C2SLoginPackets, play::C2SPlayPackets,
            status::C2SStatusPackets,
        },
        s2c::{
            config::{
                FinishConfigurationS2CConfigPacket, KnownPack, SelectKnownPacksS2CConfigPacket,
            },
            login::LoginFinishedS2CLoginPacket,
            play::{
                GameEvent, GameEventS2CPlayPacket, Gamemode, LoginS2CPlayPacket,
                PlayerPositionS2CPlayPacket, SetChunkCacheCenterS2CPlayPacket, TeleportFlags,
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
    systems::{
        events::PlayerMoveEvent,
        parameters::{Event, Param},
        typemap::TypeMap,
    },
    values::Key,
};

use super::{message::ConnectionMessage, net::ConnectionData, player::Player};

impl ConnectionData {
    pub async fn write_packet<P: PrefixedPacketEncode + Debug>(&self, packet: P) {
        let mut buf = PacketBuf::new();
        packet.encode_prefixed(&mut buf).unwrap();

        let mut len_buf = PacketBuf::new();
        VarInt::from(buf.iter().count())
            .encode(&mut len_buf)
            .unwrap();
        len_buf.write_u8s(buf.as_slice());

        let snd = self.sender.clone();
        snd.send(ConnectionMessage::SendPacket(len_buf))
            .await
            .unwrap();
    }

    pub async fn status_stage(&mut self) {
        self.read_packets(async |packet: C2SStatusPackets, this| match packet {
            C2SStatusPackets::StatusRequest(_packet) => {
                this.write_packet(
                    StatusResponse {
                        version: StatusResponseVersion {
                            name: "1.21.4".to_string(),
                            protocol: 769,
                        },
                        players: StatusResponsePlayers {
                            online: 0,
                            max: 100,
                            sample: vec![],
                        },
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
        self.read_packets(async |packet: C2SLoginPackets, this: &mut Self| {
            println!("login packet: {:?}", packet);
            match packet {
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
            }
        })
        .await;
    }

    pub async fn configuration_stage(&mut self) {
        self.read_packets(async |packet: C2SConfigPackets, this: &mut Self| {
            println!("config packet: {:?}", packet);

            match packet {
                C2SConfigPackets::CustomPayload(_packet) => {}
                C2SConfigPackets::FinishConfiguration(_packet) => {
                    this.stage = Stage::Play;
                    this.write_packet(LoginS2CPlayPacket {
                        entity: 1,
                        hardcore: false,
                        // fake dimensions so we can control client w/o extra storage
                        dims: vec![Identifier::new("wyvern", "fake")].into(),
                        max_players: VarInt::from(100),
                        view_dist: VarInt::from(2),
                        sim_dist: VarInt::from(2),
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
                        y: 5.0,
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

                    this.send_chunks().await;
                }
                C2SConfigPackets::ResourcePack(_packet) => todo!(),
                C2SConfigPackets::CookieResponse(_packet) => todo!(),
                C2SConfigPackets::Pong(_packet) => todo!(),
                C2SConfigPackets::ClientInformation(packet) => {
                    this.associated_data.render_distance = packet.info.view_distance as i32;
                }
                C2SConfigPackets::KeepAlive(_packet) => todo!(),
                C2SConfigPackets::SelectKnownPacks(_packet) => {
                    println!("pk: {:?}", _packet);
                    this.write_packet(
                        this.connected_server
                            .biomes()
                            .await
                            .to_registry_data_packet(),
                    )
                    .await;
                    this.write_packet(
                        this.connected_server
                            .damage_types()
                            .await
                            .to_registry_data_packet(),
                    )
                    .await;
                    this.write_packet(
                        this.connected_server
                            .wolf_variants()
                            .await
                            .to_registry_data_packet(),
                    )
                    .await;
                    this.write_packet(
                        this.connected_server
                            .painting_variants()
                            .await
                            .to_registry_data_packet(),
                    )
                    .await;
                    this.write_packet(
                        this.connected_server
                            .dimension_types()
                            .await
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
        self.read_packets(
            async |packet: C2SPlayPackets, this: &mut Self| match packet {
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

                    this.connected_server
                        .fire_systems({
                            let mut map = TypeMap::new();
                            map.insert(Event::<PlayerMoveEvent>::new());
                            map.insert(Param::new(this.associated_data.last_position.clone()));
                            map.insert(Param::new(Player {
                                messenger: this.sender.clone(),
                            }));
                            map
                        })
                        .await;
                }
                C2SPlayPackets::MovePlayerPosRot(packet) => {
                    this.associated_data.last_position = this
                        .associated_data
                        .last_position
                        .with_x(packet.x)
                        .with_y(packet.y)
                        .with_z(packet.z)
                        .with_pitch(packet.pitch as f64)
                        .with_yaw(packet.yaw as f64);

                    this.send_chunks().await;

                    this.connected_server
                        .fire_systems({
                            let mut map = TypeMap::new();
                            map.insert(Event::<PlayerMoveEvent>::new());
                            map.insert(Param::new(this.associated_data.last_position.clone()));
                            map.insert(Param::new(Player {
                                messenger: this.sender.clone(),
                            }));
                            map
                        })
                        .await;
                }
                C2SPlayPackets::MovePlayerRot(packet) => {
                    this.associated_data.last_position = this
                        .associated_data
                        .last_position
                        .with_pitch(packet.pitch as f64)
                        .with_yaw(packet.yaw as f64);

                    this.connected_server
                        .fire_systems({
                            let mut map = TypeMap::new();
                            map.insert(Event::<PlayerMoveEvent>::new());
                            map.insert(Param::new(this.associated_data.last_position.clone()));
                            map.insert(Param::new(Player {
                                messenger: this.sender.clone(),
                            }));
                            map
                        })
                        .await;
                }
                C2SPlayPackets::ClientInformation(packet) => {
                    this.associated_data.render_distance = packet.info.view_distance as i32;
                }
                C2SPlayPackets::PlayerInput(packet) => {
                    this.associated_data.input_flags = packet.flags;
                }
                C2SPlayPackets::ClientTickEnd(_) => {}
                packet => {
                    println!("Received unknown play packet: {:?}", packet);
                }
            },
        )
        .await;
    }
}
