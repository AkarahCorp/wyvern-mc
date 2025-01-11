use voxidian_protocol::{packet::{c2s::{handshake::{C2SHandshakePackets, IntendedStage}, status::C2SStatusPackets}, s2c::status::{StatusResponse, StatusResponsePlayers, StatusResponseS2CStatusPacket, StatusResponseVersion}, Stage}, value::{Text, TextComponent}};
use wyvern_mc::{player::proxy::Player, server::builder::ServerBuilder, systems::{events::ReceivePacketEvent, parameters::{Event, Param}}};

#[tokio::main]
async fn main() {
    let mut b = ServerBuilder::new();
    b.add_system(on_handshake);
    b.add_system(on_status);
    b.start().await;
}

async fn on_handshake(
    _event: Event<ReceivePacketEvent<C2SHandshakePackets>>,
    packet: Param<C2SHandshakePackets>,
    player: Param<Player>
) {
    println!("Received packet: {:?}", *packet);
    let C2SHandshakePackets::Intention(packet) = &*packet;

    player.set_stage(packet.intended_stage.clone().into_stage()).await;

    match packet.intended_stage {
        IntendedStage::Status => {
        
        },
        IntendedStage::Login => {},
        IntendedStage::Transfer => todo!(),
    }
}

async fn on_status(
    _event: Event<ReceivePacketEvent<C2SStatusPackets>>,
    packet: Param<C2SStatusPackets>,
    player: Param<Player>
) {
    println!("Received packet: {:?}", *packet);
    match &*packet {
        C2SStatusPackets::StatusRequest(packet) => {
            player.write_packet(
                StatusResponse {
                    version: StatusResponseVersion {
                        name: "1.21.4".to_string(),
                        protocol: 769,
                    },
                    players: StatusResponsePlayers {
                        online: 0,
                        max: 1,
                        sample: vec![],
                    },
                    desc: Text::new(),
                    favicon_png_b64: "".to_string(),
                    enforce_chat_reports: false,
                    prevent_chat_reports: true,
                }.to_packet()
            ).await;
        },
        C2SStatusPackets::PingRequest(packet) => {
            todo!();
        },
    }
}