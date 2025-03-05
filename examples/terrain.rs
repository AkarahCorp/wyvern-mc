use std::sync::{Arc, LazyLock};

use noise::{NoiseFn, Simplex};
use wyvern_mc::{
    actors::ActorResult,
    blocks::{BlockComponents, BlockState, Blocks},
    components::DataComponentHolder,
    events::{DimensionCreateEvent, PlayerJoinEvent, ServerStartEvent},
    id,
    server::Server,
    values::{Vec3, regval::DimensionType},
};

fn main() {
    env_logger::init();

    Server::builder()
        .event(on_server_start)
        .event(on_dim_init)
        .event(on_join)
        .registries(|registries| {
            registries.add_defaults();
            registries.dimension_type(
                id![minecraft:overworld],
                DimensionType::default().height(64).min_y(0),
            );
        })
        .run();
}

fn on_server_start(event: Arc<ServerStartEvent>) -> ActorResult<()> {
    event.server.create_dimension(id!(example:root))?;

    Ok(())
}

static SIMPLEX: LazyLock<Simplex> = LazyLock::new(|| Simplex::new(0));

fn on_dim_init(event: Arc<DimensionCreateEvent>) -> ActorResult<()> {
    event
        .dimension
        .set_chunk_generator(Box::new(move |chunk, x, z| {
            for x2 in 0..16 {
                for z2 in 0..16 {
                    let y = SIMPLEX.get([
                        (x2 + (x * 16)) as f64 / 100.0,
                        (z2 + (z * 16)) as f64 / 100.0,
                    ]) + 1.0;
                    let y = f64::floor(y * 16.0) as i32;

                    let new_pos = Vec3::new(x2, y, z2);
                    chunk.set_block_at(
                        new_pos,
                        BlockState::new(Blocks::GRASS_BLOCK).with(BlockComponents::SNOWY, false),
                    );

                    if SIMPLEX.get([(x2) as f64 * 100.0, (z2) as f64 * 100.0]) > 0.5 {
                        chunk.set_block_at(
                            new_pos.with_y(new_pos.y() + 1),
                            BlockState::new(Blocks::SHORT_GRASS),
                        );
                    }

                    for y in 0..y {
                        let new_pos = Vec3::new(x2, y, z2);
                        chunk.set_block_at(new_pos, BlockState::new(Blocks::DIRT));
                    }
                }
            }
        }))?;
    Ok(())
}

fn on_join(event: Arc<PlayerJoinEvent>) -> ActorResult<()> {
    event.new_dimension.set(id![example:root]);
    Ok(())
}
