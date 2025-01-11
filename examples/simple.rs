use voxidian_protocol::packet::c2s::handshake::C2SHandshakePackets;
use wyvern_mc::{server::builder::ServerBuilder, systems::{events::ReceivePacketEvent, parameters::{Event, Param}}};

#[tokio::main]
async fn main() {
    let mut b = ServerBuilder::new();
    b.add_system(on_handshake);
    b.start().await;
}

async fn on_handshake(
    _event: Event<ReceivePacketEvent<C2SHandshakePackets>>,
    packet: Param<C2SHandshakePackets>
) {
    println!("Received packet: {:?}", *packet);
}