use std::{collections::VecDeque, net::IpAddr};

use data::PlayerData;
use net::ConnectionStoppedSignal;
use tokio::{
    net::TcpStream,
    sync::mpsc::{self, Receiver, Sender},
};
use voxidian_protocol::{
    packet::{
        PacketBuf, PacketEncode, PrefixedPacketEncode, Stage,
        processing::PacketProcessing,
        s2c::play::{
            AddEntityS2CPlayPacket, ForgetLevelChunkS2CPlayPacket, GameEvent,
            GameEventS2CPlayPacket, Gamemode, PlayerPositionS2CPlayPacket,
            PlayerRotationS2CPlayPacket, RespawnS2CPlayPacket, TeleportFlags,
        },
    },
    value::{Angle, VarInt},
};
use wyvern_macros::{actor, message};

use crate::{
    dimension::Dimension,
    server::Server,
    values::{Vec2, Vec3},
};

pub mod chunkload;
pub mod data;
pub mod net;
pub mod stages;

#[actor(Player, PlayerMessage)]
pub(crate) struct ConnectionData {
    pub(crate) stream: TcpStream,
    #[allow(dead_code)]
    pub(crate) addr: IpAddr,
    pub(crate) received_bytes: VecDeque<u8>,
    pub(crate) bytes_to_send: Vec<u8>,
    pub(crate) packet_processing: PacketProcessing,
    pub(crate) signal: mpsc::Sender<ConnectionStoppedSignal>,
    pub(crate) connected_server: Server,
    pub(crate) stage: Stage,
    pub(crate) associated_data: PlayerData,
    pub(crate) sender: Sender<PlayerMessage>,
}

#[message(Player, PlayerMessage)]
impl ConnectionData {
    #[SetStage]
    pub async fn set_stage(&mut self, stage: Stage) {
        self.stage = stage;
    }

    #[GetStage]
    pub async fn get_stage(&mut self) -> Stage {
        self.stage
    }

    #[IsLoaded]
    pub async fn is_loaded_in_world(&self) -> bool {
        self.associated_data.is_loaded
    }

    #[SendPacketBuf]
    pub async fn send_packet_buf(&mut self, buf: PacketBuf) {
        self.bytes_to_send.extend(buf.iter());
    }

    #[GetServer]
    pub async fn get_server(&self) -> Server {
        self.connected_server.clone()
    }

    #[GetDimension]
    pub async fn get_dimension(&self) -> Option<Dimension> {
        self.associated_data.dimension.clone()
    }

    #[ChangeDimension]
    pub async fn change_dimension(&mut self, dimension: Dimension) {
        for chunk in self.associated_data.loaded_chunks.clone() {
            self.write_packet(ForgetLevelChunkS2CPlayPacket {
                chunk_z: chunk.y(),
                chunk_x: chunk.x(),
            })
            .await;
        }

        self.associated_data.dimension = Some(dimension.clone());
        self.associated_data.loaded_chunks.clear();
        self.associated_data.last_position = Vec3::new(0.0, 0.0, 0.0);
        self.associated_data.last_direction = Vec2::new(0.0, 0.0);

        self.write_packet(RespawnS2CPlayPacket {
            dim: self
                .connected_server
                .registries()
                .await
                .dimension_types
                .make_entry(&dimension.get_dimension_type().await.into())
                .unwrap(),
            dim_name: dimension.get_name().await.into(),
            seed: 0,
            gamemode: Gamemode::Survival,
            prev_gamemode: Gamemode::None,
            is_debug: false,
            is_flat: false,
            death_loc: None,
            portal_cooldown: VarInt::from(0),
            sea_level: VarInt::from(0),
            data_kept: 0,
        })
        .await;
        self.write_packet(GameEventS2CPlayPacket {
            event: GameEvent::WaitForChunks,
            value: 0.0,
        })
        .await;
        self.write_packet(PlayerPositionS2CPlayPacket {
            teleport_id: VarInt::from(18383),
            x: 1.0,
            y: 1.0,
            z: 1.0,
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
        })
        .await;
        self.write_packet(PlayerRotationS2CPlayPacket {
            yaw: 0.0,
            pitch: 0.0,
        })
        .await;

        for entity in dimension.get_all_entities().await {
            let position = entity.position().await;
            self.write_packet(AddEntityS2CPlayPacket {
                id: entity.entity_id().await.into(),
                uuid: *entity.uuid(),
                kind: self
                    .connected_server
                    .registries()
                    .await
                    .entity_types
                    .make_entry(&entity.entity_type().await.into())
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
}

impl Player {
    pub async fn write_packet<P: PrefixedPacketEncode + std::fmt::Debug>(&self, packet: P) {
        let mut buf = PacketBuf::new();
        packet.encode_prefixed(&mut buf).unwrap();

        let mut len_buf = PacketBuf::new();
        VarInt::from(buf.iter().count())
            .encode(&mut len_buf)
            .unwrap();

        let mut new_buf = PacketBuf::new();
        new_buf.write_u8s(len_buf.as_slice());
        new_buf.write_u8s(buf.as_slice());
        self.send_packet_buf(new_buf).await;
    }
}

impl ConnectionData {
    pub async fn write_packet<P: PrefixedPacketEncode + std::fmt::Debug>(&mut self, packet: P) {
        let mut buf = PacketBuf::new();
        packet.encode_prefixed(&mut buf).unwrap();

        let mut len_buf = PacketBuf::new();
        VarInt::from(buf.iter().count())
            .encode(&mut len_buf)
            .unwrap();

        self.bytes_to_send.extend(len_buf);
        self.bytes_to_send.extend(buf);
    }
}

#[derive(Debug)]
pub struct ConnectionWithSignal {
    pub(crate) player: Player,
    pub(crate) _signal: Receiver<ConnectionStoppedSignal>,
}

impl ConnectionWithSignal {
    pub fn lower(&self) -> Player {
        self.player.clone()
    }
}
