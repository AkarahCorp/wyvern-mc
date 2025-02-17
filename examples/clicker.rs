use std::{
    collections::HashMap,
    sync::{Arc, LazyLock, Mutex},
};

use voxidian_protocol::{
    packet::s2c::play::ScreenWindowKind,
    value::{DimEffects, DimMonsterSpawnLightLevel, DimType},
};
use wyvern_mc::{
    actors::ActorResult,
    dimension::blocks::BlockState,
    events::{
        DimensionCreateEvent, PlayerJoinEvent, RightClickEvent, ServerStartEvent, ServerTickEvent,
        SwapHandsEvent,
    },
    inventory::{Inventory, ItemStack},
    key,
    runtime::Runtime,
    server::Server,
    values::{
        Key, Uuid, Vec3,
        regval::{PaintingVariant, WolfVariant},
    },
};

static COUNTER: LazyLock<Mutex<HashMap<Uuid, i32>>> = LazyLock::new(|| Mutex::new(HashMap::new()));

#[tokio::main]
async fn main() {
    Runtime::tokio();
    Server::builder()
        .event(on_server_start)
        .event(on_dim_init)
        .event(on_join)
        .event(on_tick)
        .event(on_right_click)
        .event(on_swap_hands)
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

async fn on_server_start(event: Arc<ServerStartEvent>) -> ActorResult<()> {
    event.server.create_dimension(key!(clicker:root)).await?;

    Ok(())
}

async fn on_dim_init(event: Arc<DimensionCreateEvent>) -> ActorResult<()> {
    for x in 0..6 {
        for z in 0..6 {
            event
                .dimension
                .set_block(
                    Vec3::new(x, 0, z),
                    BlockState::new(key![minecraft:grass_block]),
                )
                .await?;
        }
    }
    Ok(())
}

async fn on_join(event: Arc<PlayerJoinEvent>) -> ActorResult<()> {
    event.new_dimension.set(key!(clicker:root));

    event
        .player
        .inventory()?
        .set_slot(40, ItemStack::new(Key::new("minecraft", "diamond")))
        .await?;
    Ok(())
}

async fn on_tick(event: Arc<ServerTickEvent>) -> ActorResult<()> {
    for player in event.server.players().await? {
        let uuid = player.uuid().await?;

        let count = {
            let mut counter = COUNTER.lock().unwrap();

            match counter.get(&uuid).copied() {
                None => {
                    counter.insert(uuid, 0);
                    0
                }
                Some(count) => count,
            }
        };

        player
            .send_action_bar(format!("Clicks: {:?}", count))
            .await?;
    }
    Ok(())
}

async fn on_right_click(event: Arc<RightClickEvent>) -> ActorResult<()> {
    let uuid = event.player.uuid().await?;
    let mut counter = COUNTER.lock().unwrap();
    if let Some(number) = counter.get_mut(&uuid) {
        *number += 1;
    };
    Ok(())
}

async fn on_swap_hands(event: Arc<SwapHandsEvent>) -> ActorResult<()> {
    event
        .player
        .open_screen(ScreenWindowKind::Generic9x1)
        .await?;
    Ok(())
}
