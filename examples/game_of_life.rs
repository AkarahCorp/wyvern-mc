use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use voxidian_protocol::value::{DimEffects, DimMonsterSpawnLightLevel, DimType};
use wyvern_mc::{
    actors::ActorResult,
    dimension::blocks::{BlockState, Blocks},
    events::{
        DimensionCreateEvent, PlayerJoinEvent, ServerStartEvent, StartBreakBlockEvent,
        SwapHandsEvent,
    },
    key,
    runtime::Runtime,
    server::Server,
    values::{
        Key, Vec3,
        regval::{PaintingVariant, WolfVariant},
    },
};

const MAX_X: usize = 50;
const MAX_Z: usize = 50;

static IS_RUNNING: AtomicBool = AtomicBool::new(false);

#[tokio::main]
async fn main() {
    env_logger::init();

    Runtime::tokio();
    Server::builder()
        .event(on_server_start)
        .event(on_dim_init)
        .event(on_join)
        .event(on_start_break)
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
    event
        .server
        .create_dimension(key!(game_of_life:overworld))
        .await?;
    Ok(())
}

async fn on_dim_init(event: Arc<DimensionCreateEvent>) -> ActorResult<()> {
    for x in 0..MAX_X {
        for z in 0..MAX_Z {
            event
                .dimension
                .set_block(
                    Vec3::new(x as i32, 0, z as i32),
                    BlockState::new(Blocks::DIRT),
                )
                .await?;
        }
    }
    Ok(())
}

async fn on_join(event: Arc<PlayerJoinEvent>) -> ActorResult<()> {
    event.new_dimension.set(key!(game_of_life:overworld));
    Ok(())
}

async fn on_start_break(event: Arc<StartBreakBlockEvent>) -> ActorResult<()> {
    let dim = event.player.dimension().await?;
    let block = event.position;

    let block_state = dim.get_block(block).await?;
    let name = block_state.name();

    if name == &Blocks::DIRT {
        dim.set_block(block, BlockState::new(Blocks::GRASS_BLOCK))
            .await?;
    } else if name == &Blocks::GRASS_BLOCK {
        dim.set_block(block, BlockState::new(Blocks::DIRT)).await?;
    }
    Ok(())
}

async fn on_swap_hands(event: Arc<SwapHandsEvent>) -> ActorResult<()> {
    IS_RUNNING.store(!IS_RUNNING.load(Ordering::Acquire), Ordering::Release);
    while IS_RUNNING.load(Ordering::Acquire) {
        run_tick(&event.player.server().await?).await?;
    }
    Ok(())
}

#[allow(clippy::needless_range_loop)]
async fn run_tick(server: &Server) -> ActorResult<()> {
    let dim = server.dimension(key![game_of_life:overworld]).await?;
    let mut copies = std::array::from_fn::<_, { MAX_X }, _>(|_| {
        std::array::from_fn::<_, { MAX_Z }, _>(|_| BlockState::new(Blocks::AIR))
    });
    let mut outputs = std::array::from_fn::<_, { MAX_X }, _>(|_| {
        std::array::from_fn::<_, { MAX_Z }, _>(|_| BlockState::new(Blocks::AIR))
    });

    for x in 0..MAX_X {
        for z in 0..MAX_Z {
            copies[x][z] = dim.get_block(Vec3::new(x as i32, 0, z as i32)).await?;
        }
    }

    for x in 0..MAX_X {
        for z in 0..MAX_Z {
            let block_state = &copies[x][z];
            let mut live_neighbors = 0;

            let min_x = if x == 0 { 0 } else { x - 1 };
            let min_z = if z == 0 { 0 } else { z - 1 };

            for dx in min_x..=(x + 1) {
                for dz in min_z..=(z + 1) {
                    if dx < MAX_X && dz < MAX_Z && (dx != x || dz != z) {
                        let d_block_state = &copies[dx][dz];
                        if d_block_state.name() == &Blocks::GRASS_BLOCK {
                            live_neighbors += 1;
                        }
                    }
                }
            }

            if block_state.name() == &Blocks::DIRT {
                if live_neighbors == 3 {
                    outputs[x][z] = BlockState::new(Blocks::GRASS_BLOCK);
                } else {
                    outputs[x][z] = BlockState::new(Blocks::DIRT);
                }
            } else if block_state.name() == &Blocks::GRASS_BLOCK {
                if live_neighbors == 2 || live_neighbors == 3 {
                    outputs[x][z] = BlockState::new(Blocks::GRASS_BLOCK);
                } else {
                    outputs[x][z] = BlockState::new(Blocks::DIRT);
                }
            }
        }
    }

    for x in 0..MAX_X {
        for z in 0..MAX_Z {
            dim.set_block(Vec3::new(x as i32, 0, z as i32), outputs[x][z].clone())
                .await?;
        }
    }

    Ok(())
}
