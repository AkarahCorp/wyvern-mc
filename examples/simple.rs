use voxidian_protocol::packet::c2s::handshake::C2SHandshakePackets;
use wyvern_mc::{
    proxy::builder::ProxyBuilder,
    server::builder::ServerBuilder,
    systems::{
        events::ReceivePacketEvent,
        parameters::{Event, Param},
    },
    values::{
        key::Key,
        regval::{painting_variant::PaintingVariant, wolf_variant::WolfVariant},
    },
};

#[tokio::main]
async fn main() {
    let mut proxy = ProxyBuilder::new();
    proxy.with_server({
        let mut b = ServerBuilder::new();
        b.add_system(example_system);
        b.modify_registries(|registries| {
            registries.wolf_variant(Key::new("minecraft", "pale"), WolfVariant {
                angry_texture: Key::empty(),
                wild_texture: Key::empty(),
                tame_texture: Key::empty(),
                biomes: Vec::new(),
            });
            registries.painting_variant(Key::new("minecraft", "something_idk"), PaintingVariant {
                asset: Key::empty(),
                width: 1,
                height: 1,
            });
        });
        b
    });
    proxy.start_all().await;
}

async fn example_system(
    _event: Event<ReceivePacketEvent<C2SHandshakePackets>>,
    packet: Param<C2SHandshakePackets>,
) {
    println!("packet: {:?}", *packet);
}
