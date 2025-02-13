use std::sync::LazyLock;

use noise::{NoiseFn, Simplex};
use rand::Rng;
use tokio::runtime::Builder;
use voxidian_protocol::{
    packet::Stage,
    value::{DimEffects, DimMonsterSpawnLightLevel, DimType},
};
use wyvern_mc::{
    components::ComponentHolder,
    dimension::{
        blocks::{BlockState, Blocks},
        chunk::Chunk,
        properties::BlockComponents,
    },
    events::{DimensionCreateEvent, PlayerCommandEvent, ServerStartEvent, ServerTickEvent},
    inventory::{Inventory, ItemComponents, ItemStack},
    key,
    proxy::ProxyBuilder,
    runtime::Runtime,
    server::ServerBuilder,
    values::{
        Key, Vec3,
        regval::{PaintingVariant, WolfVariant},
    },
};

fn main() {
    env_logger::init();

    Runtime::tokio();

    log::info!(
        "Running proxy with {:?} worker threads:",
        std::thread::available_parallelism().unwrap()
    );

    let rt = Builder::new_multi_thread()
        .worker_threads(std::thread::available_parallelism().unwrap().into())
        .enable_io()
        .enable_time()
        .build()
        .unwrap();

    rt.block_on(main_rt());
}

async fn main_rt() {
    let mut proxy = ProxyBuilder::new();
    proxy.with_server({
        let mut b = ServerBuilder::new();
        b.on_event(on_server_start);
        b.on_event(on_server_tick);
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
        for x in 1..100 {
            for y in 1..10 {
                for z in 1..100 {
                    let value = rand::rng().random_bool(0.5);
                    event
                        .player
                        .get_dimension()
                        .await
                        .unwrap()
                        .set_block(
                            Vec3::new(x, y, z),
                            BlockState::new(Key::new("minecraft", "grass_block"))
                                .with(&BlockComponents::SNOWY, value),
                        )
                        .await;
                }
            }
        }
    }

    if event.command == "rootdir" {
        let dimension = event
            .player
            .get_server()
            .await
            .dimension(key!(wyvern:root))
            .await
            .unwrap();
        event.player.change_dimension(dimension).await;
    }

    if event.command == "altdir" {
        let dimension = event
            .player
            .get_server()
            .await
            .dimension(key!(example:alternate))
            .await
            .unwrap();
        event.player.change_dimension(dimension).await;
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
                    chunk.set_block_at(new_pos, BlockState::new(Blocks::BLACKSTONE));
                }
            }
        })
        .await;
}

async fn on_server_tick(event: ServerTickEvent) {
    for dim in event.server.get_all_dimensions().await {
        for mut entity in dim.get_all_entities().await {
            let new_pos = Vec3::new(
                rand::random::<f64>() * 128.0,
                rand::random::<f64>() * 16.0,
                rand::random::<f64>() * 128.0,
            );
            entity.teleport(new_pos).await;
        }
    }

    for player in event.server.all_players().await {
        if player.get_stage().await == Stage::Play {
            player
                .get_inventory()
                .set_slot(
                    36,
                    ItemStack::new(Key::new("minecraft", "stone"))
                        .with(&ItemComponents::MAX_DAMAGE, 10)
                        .with(&ItemComponents::DAMAGE, 1)
                        .with(
                            &ItemComponents::ITEM_MODEL,
                            Key::constant("minecraft", "stone"),
                        ),
                )
                .await;

            player
                .get_inventory()
                .set_slot(
                    37,
                    ItemStack::new(Key::new("minecraft", "diamond_sword"))
                        .with(&ItemComponents::MAX_DAMAGE, 20)
                        .with(&ItemComponents::DAMAGE, 6)
                        .with(
                            &ItemComponents::ITEM_MODEL,
                            Key::constant("minecraft", "diamond_sword"),
                        ),
                )
                .await;
        }
    }
}

async fn on_server_start(event: ServerStartEvent) {
    event.server.create_dimension(key!(example:alternate)).await;

    for _dimension in event.server.get_all_dimensions().await {
        for _ in 1..100000000 {
            Runtime::yield_now().await;
        }
    }
}
