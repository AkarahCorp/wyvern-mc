use std::sync::Arc;

use wyvern_mc::{
    actors::ActorResult,
    blocks::{BlockComponents, BlockState},
    components::DataComponentHolder,
    events::{DimensionCreateEvent, PlayerJoinEvent, ServerStartEvent},
    id,
    server::Server,
    values::Vec3,
};

fn main() {
    env_logger::init();

    Server::builder()
        .event(on_server_start)
        .event(on_dim_init)
        .event(on_join)
        .registries(|registries| {
            registries.add_defaults();
        })
        .run();
}

fn on_server_start(event: Arc<ServerStartEvent>) -> ActorResult<()> {
    event.server.create_dimension(id!(example:root))?;

    Ok(())
}

fn on_dim_init(event: Arc<DimensionCreateEvent>) -> ActorResult<()> {
    for x in 0..10 {
        for z in 0..10 {
            event.dimension.set_block(
                Vec3::new(x, 0, z),
                BlockState::new(id![minecraft:grass_block]).with(BlockComponents::SNOWY, false),
            )?;
        }
    }
    Ok(())
}

fn on_join(event: Arc<PlayerJoinEvent>) -> ActorResult<()> {
    event.new_dimension.set(id![example:root]);
    Ok(())
}
