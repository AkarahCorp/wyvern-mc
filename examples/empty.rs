use std::sync::Arc;

use wyvern_mc::{
    actors::ActorResult,
    blocks::{BlockComponents, BlockState},
    components::DataComponentHolder,
    events::{DimensionCreateEvent, PlayerJoinEvent, ServerStartEvent},
    id,
    server::Server,
    values::{
        Id, Vec3,
        regval::{DimensionType, PaintingVariant, WolfVariant},
    },
};

fn main() {
    env_logger::init();

    Server::builder()
        .event(on_server_start)
        .event(on_dim_init)
        .event(on_join)
        .registries(|registries| {
            registries.wolf_variant(
                Id::new("minecraft", "pale"),
                WolfVariant {
                    angry_texture: Id::empty(),
                    wild_texture: Id::empty(),
                    tame_texture: Id::empty(),
                    biomes: Vec::new(),
                },
            );
            registries.painting_variant(
                Id::new("minecraft", "empty_painting"),
                PaintingVariant {
                    asset: Id::empty(),
                    width: 1,
                    height: 1,
                },
            );
            registries.dimension_type(
                Id::new("minecraft", "overworld"),
                DimensionType::default().min_y(0).height(256),
            );
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
