use std::{sync::Arc, time::Duration};

use datafix::serialization::{Codec, DefaultCodec};
use wyvern_mc::{
    actors::ActorResult,
    blocks::{BlockState, Structure},
    components::DataComponentHolder,
    datatypes::{
        gamemode::Gamemode,
        nbt::{Nbt, NbtCompound, NbtOps},
    },
    entities::{AttributeContainer, Attributes},
    events::{
        BreakBlockEvent, DimensionCreateEvent, PlaceBlockEvent, PlayerJoinEvent, ServerStartEvent,
    },
    player::PlayerComponents,
    runtime::Runtime,
    server::Server,
    values::{DVec3, IVec3, id},
};

fn main() {
    env_logger::init();

    Server::builder()
        .event(on_server_start)
        .event(on_dim_init)
        .event(on_join)
        .event(on_break)
        .event(on_place)
        .registries(|registries| {
            registries.add_defaults();
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

async fn on_dim_init(event: Arc<DimensionCreateEvent>) -> ActorResult<()> {
    let bytes = include_bytes!("./assets/h.nbt").to_vec();
    let nbt = Nbt::new(NbtCompound::try_from(bytes).unwrap());
    let structure = Structure::codec().decode_start(&NbtOps, &nbt).unwrap();

    structure.place(event.dimension.clone(), IVec3::new(0, 0, 0))?;
    Ok(())
}

async fn on_join(event: Arc<PlayerJoinEvent>) -> ActorResult<()> {
    event.new_dimension.set(id![example:root]);
    event.player.set_gamemode(Gamemode::Survival)?;
    event.player.set(
        PlayerComponents::TELEPORT_POSITION,
        DVec3::new(0.0, 1.0, 0.0),
    )?;

    Runtime::spawn_task(async move {
        std::thread::sleep(Duration::from_millis(10000));
        event.player.set(
            PlayerComponents::ATTRIBUTES,
            AttributeContainer::new()
                .with(Attributes::MINING_EFFICIENCY, 100.0)
                .with(Attributes::MAX_HEALTH, 40.0),
        )?;
        Ok(())
    });
    Ok(())
}

async fn on_break(event: Arc<BreakBlockEvent>) -> ActorResult<()> {
    let dim = event.player.dimension()?;
    dim.set_block(event.position, event.old_block.clone())?;
    Ok(())
}

async fn on_place(event: Arc<PlaceBlockEvent>) -> ActorResult<()> {
    let dim = event.player.dimension()?;
    dim.set_block(event.position, BlockState::new(id![minecraft:air]))?;
    Ok(())
}
