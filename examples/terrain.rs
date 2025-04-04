use std::sync::{Arc, LazyLock};

use noise::{NoiseFn, Simplex};
use wyvern_mc::{
    actors::ActorResult,
    blocks::{BlockComponents, BlockState, Blocks},
    components::DataComponentHolder,
    datatypes::{gamemode::Gamemode, regval::DimensionType},
    dimension::chunk::Chunk,
    events::{DimensionCreateEvent, PlayerJoinEvent, ServerStartEvent},
    player::PlayerComponents,
    server::{Server, registries::RegistryKeys},
    values::{IVec3, id},
};

fn main() {
    env_logger::init();

    Server::builder()
        .event(on_server_start)
        .event(on_dim_init)
        .event(on_join)
        .registries(|registries| {
            registries.get_mut(RegistryKeys::DIMENSION_TYPE).insert(
                id![minecraft:overworld],
                DimensionType::default().height(64).min_y(0),
            );
        })
        .run();
}

async fn on_server_start(event: Arc<ServerStartEvent>) -> ActorResult<()> {
    event
        .server
        .create_dimension(id!(example:root), id![minecraft:overworld])?;
    event.server.set_default_dimension(id![example:root])?;

    Ok(())
}

static SIMPLEX: LazyLock<Simplex> = LazyLock::new(|| Simplex::new(0));

async fn on_dim_init(event: Arc<DimensionCreateEvent>) -> ActorResult<()> {
    event
        .dimension
        .set_chunk_generator(move |chunk: &mut Chunk, x, z| {
            for x2 in 0..16 {
                for z2 in 0..16 {
                    let y = SIMPLEX.get([
                        (x2 + (x * 16)) as f64 / 100.0,
                        (z2 + (z * 16)) as f64 / 100.0,
                    ]) + 1.0;
                    let y = f64::floor(y * 16.0) as i32;

                    let new_pos = IVec3::new(x2, y, z2);
                    chunk.set_block_at(
                        new_pos,
                        &BlockState::new(Blocks::GRASS_BLOCK).with(BlockComponents::SNOWY, false),
                    );

                    if SIMPLEX.get([(x2) as f64 * 100.0, (z2) as f64 * 100.0]) > 0.5 {
                        chunk.set_block_at(
                            new_pos.with_y(new_pos[1] + 1),
                            &BlockState::new(Blocks::SHORT_GRASS),
                        );
                    }

                    for y in 0..y {
                        let new_pos = IVec3::new(x2, y, z2);
                        chunk.set_block_at(new_pos, &BlockState::new(Blocks::DIRT));
                    }
                }
            }
        })?;
    Ok(())
}

async fn on_join(event: Arc<PlayerJoinEvent>) -> ActorResult<()> {
    event.new_dimension.set(id![example:root]);
    event
        .player
        .set(PlayerComponents::GAMEMODE, Gamemode::Creative)?;
    Ok(())
}
