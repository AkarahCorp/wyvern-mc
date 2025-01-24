use voxidian_protocol::{
    packet::c2s::handshake::C2SHandshakePackets,
    value::{DimEffects, DimMonsterSpawnLightLevel, DimType},
};
use wyvern_mc::{
    dimension::blocks::BlockState,
    player::player::Player,
    proxy::ProxyBuilder,
    server::{Server, ServerBuilder},
    systems::{
        events::{PlayerMoveEvent, ReceivePacketEvent, ServerTickEvent},
        parameters::{Event, Param},
    },
    values::{
        Key, Position,
        regval::{PaintingVariant, WolfVariant},
    },
};

#[tokio::main]
async fn main() {
    let mut proxy = ProxyBuilder::new();
    proxy.with_server({
        let mut b = ServerBuilder::new();
        b.add_system(example_system);
        b.add_system(on_tick);
        b.add_system(on_move);
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
                max_y: 256,
                logical_height: 256,
                height: 256,
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

async fn on_tick(_event: Event<ServerTickEvent>, server: Param<Server>) {}

async fn on_move(
    _event: Event<PlayerMoveEvent>,
    player: Param<Player>,
    pos: Param<Position<f64, f64>>,
) {
    let dim = player.get_dimension().await;
    dim.set_block_at(
        pos.with_y(*pos.y() + 3.0).map_angled(|x| *x as i32, |x| ()),
        BlockState::from_protocol_id(1),
    )
    .await;
}
