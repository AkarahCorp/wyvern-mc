use std::sync::Arc;

use wyvern_mc::{
    actors::ActorResult,
    blocks::{BlockComponents, BlockState},
    components::DataComponentHolder,
    entities::EntityComponents,
    events::{DimensionCreateEvent, PlayerAttackEntityEvent, PlayerJoinEvent, ServerStartEvent},
    id,
    server::Server,
    values::{Texts, Vec3},
};

fn main() {
    env_logger::init();

    Server::builder()
        .event(on_server_start)
        .event(on_dim_init)
        .event(on_join)
        .event(on_attack)
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

    let mut entity = event.dimension.spawn_entity(id![minecraft:zombie])?;
    entity.set(EntityComponents::POSITION, Vec3::new(3.0, 1.0, 3.0))?;
    Ok(())
}

fn on_join(event: Arc<PlayerJoinEvent>) -> ActorResult<()> {
    event.new_dimension.set(id![example:root]);
    Ok(())
}

fn on_attack(event: Arc<PlayerAttackEntityEvent>) -> ActorResult<()> {
    event
        .player
        .send_message(Texts::literal("HI YOU HIT AN ENTITY WOW"))?;
    Ok(())
}
