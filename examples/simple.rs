use std::sync::{Arc, LazyLock};

use noise::{NoiseFn, Simplex};
use tokio::runtime::Builder;
use voxidian_protocol::{
    packet::Stage,
    value::{DimEffects, DimMonsterSpawnLightLevel, DimType},
};
use wyvern_mc::{
    actors::ActorResult,
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
    runtime::Runtime,
    server::Server,
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
    Server::builder()
        .event(on_server_start)
        .event(on_server_tick)
        .event(dim_init)
        .event(on_command)
        .event(on_drop_item)
        .event(on_place)
        .event(on_break)
        .event(on_chat)
        .event(on_join)
        .registries(|registries| {
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
        })
        .run()
        .await;
}

static SIMPLEX: LazyLock<Simplex> = LazyLock::new(|| Simplex::new(0));

async fn on_command(event: Arc<PlayerCommandEvent>) -> ActorResult<()> {
    if event.command.as_str() == "overload" {
        let event = event.clone();
        Runtime::spawn(async move {
            let state = BlockState::new(Key::new("minecraft", "grass_block"))
                .with(&BlockComponents::SNOWY, false);
            let dim = event.player.dimension().await.unwrap();
            for x in 1..100 {
                for y in 1..10 {
                    for z in 1..100 {
                        let _ = dim.set_block(Vec3::new(x, y, z), state.clone()).await;
                    }
                }
            }
        });
    }

    if event.command == "rootdir" {
        let dimension = event
            .player
            .server()
            .await?
            .dimension(key!(wyvern:root))
            .await?;
        event.player.set_dimension(dimension).await?;
    }

    if event.command == "altdir" {
        let dimension = event
            .player
            .server()
            .await?
            .dimension(key!(example:alternate))
            .await?;
        event.player.set_dimension(dimension).await?;
    }

    Ok(())
}

async fn dim_init(event: Arc<DimensionCreateEvent>) -> ActorResult<()> {
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
                    chunk.set_block_at(new_pos, BlockState::new(Blocks::GRASS_BLOCK));
                }
            }
        })
        .await;

    Ok(())
}

async fn on_server_tick(event: Arc<ServerTickEvent>) -> ActorResult<()> {
    for dim in event.server.dimensions().await? {
        for mut entity in dim.entities().await? {
            let new_pos = Vec3::new(
                rand::random::<f64>() * 128.0,
                rand::random::<f64>() * 16.0,
                rand::random::<f64>() * 128.0,
            );
            entity.teleport(new_pos).await?;
        }
    }

    for player in event.server.players().await? {
        if player.stage().await? == Stage::Play {
            player
                .inventory()?
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
                .await?;

            player
                .inventory()?
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
                .await?;
        }
    }

    Ok(())
}

async fn on_server_start(event: Arc<ServerStartEvent>) -> ActorResult<()> {
    event.server.create_dimension(key!(example:root)).await?;
    event
        .server
        .create_dimension(key!(example:alternate))
        .await?;

    Ok(())
}

async fn on_drop_item(event: Arc<DropItemEvent>) -> ActorResult<()> {
    event
        .player
        .send_message("You dropped an item, wow!".to_string())
        .await?;

    Ok(())
}

async fn on_place(event: Arc<PlaceBlockEvent>) -> ActorResult<()> {
    event
        .player
        .send_message("You placed an item, wow!".to_string())
        .await?;
    Ok(())
}

async fn on_break(event: Arc<BreakBlockEvent>) -> ActorResult<()> {
    event
        .player
        .send_message("You broke an item, wow!".to_string())
        .await?;
    Ok(())
}

async fn on_chat(event: Arc<ChatMessageEvent>) -> ActorResult<()> {
    log::error!("Sending message...");
    for player in Server::get()?.players().await? {
        player
            .send_message(format!(
                "{}: {}",
                event.player.username().await?,
                event.message
            ))
            .await?;
    }
    Ok(())
}

async fn on_join(event: Arc<PlayerJoinEvent>) -> ActorResult<()> {
    event.new_dimension.set(key!(example:root));
    Ok(())
}
