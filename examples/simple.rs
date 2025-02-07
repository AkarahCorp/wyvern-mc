use std::{sync::LazyLock, time::Instant};

use noise::{NoiseFn, Simplex};
use rand::Rng;
use voxidian_protocol::value::{DimEffects, DimMonsterSpawnLightLevel, DimType};
use wyvern_mc::{
    dimension::{blocks::BlockState, chunk::Chunk, properties::Properties},
    events::{DimensionCreateEvent, PlayerCommandEvent},
    proxy::ProxyBuilder,
    server::ServerBuilder,
    values::{
        Key, Vec3,
        regval::{PaintingVariant, WolfVariant},
    },
};

#[tokio::main]
async fn main() {
    let mut proxy = ProxyBuilder::new();
    proxy.with_server({
        let mut b = ServerBuilder::new();
        b.on_event(dim_init);
        b.on_event(on_command);
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
                min_y: -32,
                max_y: 32,
                logical_height: 64,
                height: 64,
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

static SIMPLEX: LazyLock<Simplex> = LazyLock::new(|| Simplex::new(0));

async fn on_command(event: PlayerCommandEvent) {
    if event.command.as_str() == "overload" {
        let start = Instant::now();
        for x in 1..100 {
            for y in 1..10 {
                for z in 1..100 {
                    let value = rand::thread_rng().gen_bool(0.5);
                    event
                        .player
                        .get_dimension()
                        .await
                        .set_block(
                            Vec3::new(x, y, z),
                            BlockState::new(Key::new("minecraft", "grass_block"))
                                .with_property(Properties::SNOWY, value),
                        )
                        .await;
                }
            }
        }
        let end = Instant::now();
        println!("Time: {:?}", end - start);
    }
}

async fn dim_init(event: DimensionCreateEvent) {
    event
        .dimension
        .set_chunk_generator(|chunk: &mut Chunk, x, z| {
            if x < 0 {
                return;
            }
            if z < 0 {
                return;
            }
            for x2 in 0..16 {
                for z2 in 0..16 {
                    let y = SIMPLEX.get([
                        (x2 + (x * 16)) as f64 / 100.0,
                        (z2 + (z * 16)) as f64 / 100.0,
                    ]) + 1.0;

                    let new_pos = Vec3::new(x2, f64::floor(y * -16.0 + 8.0) as i32, z2);
                    chunk.set_block_at(
                        new_pos,
                        BlockState::new(Key::new("minecraft", "grass_block"))
                            .with_property(Properties::SNOWY, false),
                    );
                }
            }
        })
        .await;
}
