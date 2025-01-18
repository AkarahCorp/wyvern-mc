use std::fmt::Debug;

use voxidian_protocol::{
    packet::{
        PacketBuf, PacketEncode, PrefixedPacketEncode, Stage,
        c2s::{config::C2SConfigPackets, login::C2SLoginPackets, status::C2SStatusPackets},
        s2c::{
            config::{
                FinishConfigurationS2CConfigPacket, KnownPack, SelectKnownPacksS2CConfigPacket,
            },
            login::LoginFinishedS2CLoginPacket,
            play::{Gamemode, LoginS2CPlayPacket, PlayerPositionS2CPlayPacket, TeleportFlags},
            status::{
                PongResponseS2CStatusPacket, StatusResponse, StatusResponsePlayers,
                StatusResponseVersion,
            },
        },
    },
    registry::RegEntry,
    value::{Identifier, LengthPrefixHashMap, Text, VarInt},
};

use super::{message::ConnectionMessage, net::ConnectionData};

impl ConnectionData {
    pub fn write_packet<P: PrefixedPacketEncode + Debug>(&mut self, packet: P) {
        println!("Sending: {:?}", packet);
        let mut buf = PacketBuf::new();
        packet.encode_prefixed(&mut buf).unwrap();

        let mut len_buf = PacketBuf::new();
        VarInt::from(buf.iter().count())
            .encode(&mut len_buf)
            .unwrap();

        let snd = self.sender.clone();
        tokio::spawn(async move {
            snd.send(ConnectionMessage::SendPacket(len_buf))
                .await
                .unwrap();
            snd.send(ConnectionMessage::SendPacket(buf)).await.unwrap();
        });
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
                );
            }
            C2SStatusPackets::PingRequest(packet) => {
                this.write_packet(PongResponseS2CStatusPacket {
                    timestamp: packet.timestamp,
                });
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
                    });
                }
                C2SLoginPackets::Key(_packet) => todo!(),
                C2SLoginPackets::Hello(packet) => {
                    this.associated_data.username = packet.username.clone();
                    this.associated_data.uuid = packet.uuid.clone();
                    this.write_packet(LoginFinishedS2CLoginPacket {
                        uuid: packet.uuid,
                        username: packet.username,
                        props: LengthPrefixHashMap::new(),
                    });
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
                        entity: 0,
                        hardcore: false,
                        // fake dimensions so we can control client w/o extra storage
                        dims: vec![Identifier::new("wyvern", "fake")].into(),
                        max_players: VarInt::from(0),
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
                        gamemode: Gamemode::Survival,
                        old_gamemode: Gamemode::None,
                        is_debug: false,
                        is_flat: false,
                        death_loc: None,
                        portal_cooldown: VarInt::from(0),
                        sea_level: VarInt::from(64),
                        enforce_chat_reports: false,
                    });
                    this.write_packet(PlayerPositionS2CPlayPacket {
                        teleport_id: VarInt::from(0),
                        x: 0.0,
                        y: 0.0,
                        z: 0.0,
                        vx: 0.0,
                        vy: 0.0,
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
                    });
                }
                C2SConfigPackets::ResourcePack(_packet) => todo!(),
                C2SConfigPackets::CookieResponse(_packet) => todo!(),
                C2SConfigPackets::Pong(_packet) => todo!(),
                C2SConfigPackets::ClientInformation(_packet) => {}
                C2SConfigPackets::KeepAlive(_packet) => todo!(),
                C2SConfigPackets::SelectKnownPacks(_packet) => {
                    println!("pk: {:?}", _packet);
                    this.write_packet(
                        this.connected_server
                            .biomes()
                            .await
                            .to_registry_data_packet(),
                    );
                    this.write_packet(
                        this.connected_server
                            .damage_types()
                            .await
                            .to_registry_data_packet(),
                    );
                    this.write_packet(
                        this.connected_server
                            .wolf_variants()
                            .await
                            .to_registry_data_packet(),
                    );
                    this.write_packet(
                        this.connected_server
                            .painting_variants()
                            .await
                            .to_registry_data_packet(),
                    );
                    this.write_packet(
                        this.connected_server
                            .dimension_types()
                            .await
                            .to_registry_data_packet(),
                    );
                    this.write_packet(FinishConfigurationS2CConfigPacket);
                }
            }
        })
        .await;
    }
}
