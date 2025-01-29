use std::{hash::DefaultHasher, sync::LazyLock};

use noise::{
    NoiseFn, Simplex, Vector2, core::open_simplex::open_simplex_2d,
    permutationtable::PermutationTable,
};
use voxidian_protocol::{
    packet::s2c::play::AwardStatsS2CPlayPacket,
    value::{DimEffects, DimMonsterSpawnLightLevel, DimType},
};
use wyvern_mc::{
    dimension::{Dimension, blocks::BlockState, chunk::ChunkSection},
    player::Player,
    proxy::ProxyBuilder,
    server::ServerBuilder,
    systems::{
        events::{ChunkLoadEvent, DimensionCreateEvent, PlayerMoveEvent},
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
        b.add_system(on_move);
        b.add_system(dim_init);
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

async fn on_move(
    _event: Event<PlayerMoveEvent>,
    player: Param<Player>,
    pos: Param<Position<f64, f64>>,
) {
}

static SIMPLEX: LazyLock<Simplex> = LazyLock::new(|| Simplex::new(0));

async fn dim_init(_event: Event<DimensionCreateEvent>, dim: Param<Dimension>) {
    dim.set_chunk_generator(|chunk: &mut ChunkSection, x, z| {
        if x < 0 {
            return;
        }
        if z < 0 {
            return;
        }
        for x2 in 0..16 {
            for z2 in 0..16 {
                let y = SIMPLEX.get([(x2 + (x * 16)) as f64 / 50.0, (z2 + (z * 16)) as f64 / 50.0]);

                let new_pos = Position::new(x2 as usize, (y * 16.0) as usize, z2 as usize);
                chunk.set_block_at(new_pos, BlockState::from_protocol_id(1));
            }
        }
    })
    .await;
}
