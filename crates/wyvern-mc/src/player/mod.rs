use std::{collections::VecDeque, net::IpAddr};

use data::PlayerData;
use net::ConnectionStoppedSignal;
use tokio::{
    net::TcpStream,
    sync::mpsc::{self, Receiver, Sender},
};
use voxidian_protocol::{
    packet::{PacketBuf, PacketEncode, PrefixedPacketEncode, Stage, processing::PacketProcessing},
    value::VarInt,
};

use crate::{dimension::Dimension, server::Server};

pub mod chunkload;
pub mod data;
pub mod net;
pub mod stages;

#[crate::actor(Player, PlayerMessage)]
pub struct ConnectionData {
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

#[crate::message(Player, PlayerMessage)]
impl ConnectionData {
    #[SetStage]
    pub async fn set_stage(&mut self, stage: Stage) {
        self.stage = stage;
    }

    #[GetStage]
    pub async fn get_stage(&mut self) -> Stage {
        self.stage.clone()
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
    pub async fn get_dimension(&self) -> Dimension {
        self.associated_data.dimension.clone().unwrap()
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
        new_buf.write_u8s(&len_buf.as_slice());
        new_buf.write_u8s(&buf.as_slice());
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
