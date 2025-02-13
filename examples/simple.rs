use std::sync::{Arc, LazyLock};

use noise::{NoiseFn, Simplex};
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
    events::{
        BreakBlockEvent, ChatMessageEvent, DimensionCreateEvent, DropItemEvent, PlaceBlockEvent,
        PlayerCommandEvent, PlayerJoinEvent, ServerStartEvent, ServerTickEvent,
    },
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
        b.on_event(on_drop_item);
        b.on_event(on_place);
        b.on_event(on_break);
        b.on_event(on_chat);
        b.on_event(on_join);
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

async fn on_command(event: Arc<PlayerCommandEvent>) {
    if event.command.as_str() == "overload" {
        let event = event.clone();
        Runtime::spawn(async move {
            let state = BlockState::new(Key::new("minecraft", "grass_block"))
                .with(&BlockComponents::SNOWY, false);
            let dim = event.player.get_dimension().await.unwrap();
            for x in 1..100 {
                for y in 1..10 {
                    for z in 1..100 {
                        dim.set_block(Vec3::new(x, y, z), state.clone()).await;
                        Runtime::yield_now().await;
                        Runtime::yield_now().await;
                        Runtime::yield_now().await;
                        Runtime::yield_now().await;
                        Runtime::yield_now().await;
                        Runtime::yield_now().await;
                        Runtime::yield_now().await;
                        Runtime::yield_now().await;
                        Runtime::yield_now().await;
                        Runtime::yield_now().await;
                    }
                }
            }
        });
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

async fn dim_init(event: Arc<DimensionCreateEvent>) {
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

async fn on_server_tick(event: Arc<ServerTickEvent>) {
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

async fn on_server_start(event: Arc<ServerStartEvent>) {
    event.server.create_dimension(key!(example:root)).await;
    event.server.create_dimension(key!(example:alternate)).await;
}

async fn on_drop_item(event: Arc<DropItemEvent>) {
    event.player.send_message("You dropped an item, wow!").await;
}

async fn on_place(event: Arc<PlaceBlockEvent>) {
    event.player.send_message("You placed an item, wow!").await;
}

async fn on_break(event: Arc<BreakBlockEvent>) {
    event.player.send_message("You broke an item, wow!").await;
}

async fn on_chat(event: Arc<ChatMessageEvent>) {
    for player in event.player.get_server().await.all_players().await {
        player
            .send_message(&format!(
                "{}: {}",
                event.player.username().await,
                event.message
            ))
            .await;
    }
}

async fn on_join(event: Arc<PlayerJoinEvent>) {
    event.new_dimension.set(key!(example:root));
}
