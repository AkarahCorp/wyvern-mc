use voxidian_protocol::{
    packet::c2s::handshake::C2SHandshakePackets,
    value::{DimEffects, DimMonsterSpawnLightLevel, DimType},
};
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
            registries.dimension_type(Key::new("minecraft", "overworld"), DimType {
                fixed_time: None,
                has_skylight: true,
                has_ceiling: false,
                ultrawarm: false,
                natural: true,
                coordinate_scale: 1.0,
                bed_works: true,
                respawn_anchor_works: true,
                min_y: 0,
                max_y: 16,
                logical_height: 16,
                height: 16,
                infiniburn: "#minecraft:overworld_infiniburn".to_string(),
                effects: DimEffects::Overworld,
                ambient_light: 15.0,
                piglin_safe: false,
                has_raids: true,
                monster_spawn_light_level: DimMonsterSpawnLightLevel::Constant(0),
                monster_spawn_block_light_limit: 0,
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
