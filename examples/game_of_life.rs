use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Instant,
};

use wyvern_mc::{
    actors::ActorResult,
    blocks::{BlockState, Blocks},
    datatypes::regval::DimensionType,
    events::{
        DimensionCreateEvent, PlayerJoinEvent, ServerStartEvent, StartBreakBlockEvent,
        SwapHandsEvent,
    },
    runtime::Runtime,
    server::{Server, registries::RegistryKeys},
    values::{Id, Vec3, id},
};

const MAX_X: usize = 50;
const MAX_Z: usize = 50;

static IS_RUNNING: AtomicBool = AtomicBool::new(false);

fn main() {
    env_logger::init();

    Server::builder()
        .event(on_server_start)
        .event(on_dim_init)
        .event(on_join)
        .event(on_start_break)
        .event(on_swap_hands)
        .registries(|registries| {
            registries.get_mut(RegistryKeys::DIMENSION_TYPE).insert(
                Id::new("minecraft", "overworld"),
                DimensionType::default().min_y(0).height(16),
            );
        })
        .run();
}

async fn on_server_start(event: Arc<ServerStartEvent>) -> ActorResult<()> {
    event.server.create_dimension(id!(game_of_life:overworld))?;
    Ok(())
}

async fn on_dim_init(event: Arc<DimensionCreateEvent>) -> ActorResult<()> {
    for x in 0..MAX_X {
        for z in 0..MAX_Z {
            event.dimension.set_block(
                Vec3::new(x as i32, 0, z as i32),
                BlockState::new(Blocks::DIRT),
            )?;
        }
    }
    Ok(())
}

async fn on_join(event: Arc<PlayerJoinEvent>) -> ActorResult<()> {
    event.new_dimension.set(id!(game_of_life:overworld));
    Ok(())
}

async fn on_start_break(event: Arc<StartBreakBlockEvent>) -> ActorResult<()> {
    let dim = event.player.dimension()?;
    let block = event.position;

    let block_state = dim.get_block(block)?;
    let name = block_state.name();

    if name == &Blocks::DIRT {
        dim.set_block(block, BlockState::new(Blocks::GRASS_BLOCK))?;
    } else if name == &Blocks::GRASS_BLOCK {
        dim.set_block(block, BlockState::new(Blocks::DIRT))?;
    }
    Ok(())
}

async fn on_swap_hands(event: Arc<SwapHandsEvent>) -> ActorResult<()> {
    IS_RUNNING.store(!IS_RUNNING.load(Ordering::Acquire), Ordering::Release);
    while IS_RUNNING.load(Ordering::Acquire) {
        run_tick(&event.player.server()?).await?;
        Runtime::yield_now().await;
    }
    Ok(())
}

#[allow(clippy::needless_range_loop)]
async fn run_tick(server: &Server) -> ActorResult<()> {
    let start = Instant::now();
    let dim = server.dimension(id![game_of_life:overworld])?;
    let mut copies = std::array::from_fn::<_, { MAX_X }, _>(|_| {
        std::array::from_fn::<_, { MAX_Z }, _>(|_| BlockState::new(Blocks::AIR))
    });
    let mut outputs = std::array::from_fn::<_, { MAX_X }, _>(|_| {
        std::array::from_fn::<_, { MAX_Z }, _>(|_| BlockState::new(Blocks::AIR))
    });

    for x in 0..MAX_X {
        for z in 0..MAX_Z {
            copies[x][z] = dim.get_block(Vec3::new(x as i32, 0, z as i32))?;
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
    let end1 = Instant::now();

    for x in 0..MAX_X {
        for z in 0..MAX_Z {
            dim.set_block(Vec3::new(x as i32, 0, z as i32), outputs[x][z].clone())?;
        }
    }
    let end2 = Instant::now();

    log::error!("{:?}", (end2 - start, end1 - start, end2 - end1));
    Ok(())
}
