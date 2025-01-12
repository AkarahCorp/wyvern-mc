use voxidian_protocol::{packet::{c2s::{handshake::{C2SHandshakePackets, IntendedStage}, status::C2SStatusPackets}, s2c::status::{PongResponseS2CStatusPacket, StatusResponse, StatusResponsePlayers, StatusResponseS2CStatusPacket, StatusResponseVersion}, Stage}, value::{Text, TextComponent}};
use wyvern_mc::{player::proxy::Player, server::builder::ServerBuilder, systems::{events::ReceivePacketEvent, parameters::{Event, Param}}};

#[tokio::main]
async fn main() {
    let mut b = ServerBuilder::new();
    b.start().await;
}