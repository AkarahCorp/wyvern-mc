use std::{sync::Arc, time::Duration};

use wyvern_mc::{
    actors::ActorResult,
    blocks::{BlockComponents, BlockState},
    components::DataComponentHolder,
    datatypes::gamemode::Gamemode,
    entities::{AttributeContainer, Attributes},
    events::{
        BreakBlockEvent, DimensionCreateEvent, PlaceBlockEvent, PlayerJoinEvent, ServerStartEvent,
    },
    inventory::Inventory,
    item::{ItemComponents, ItemStack},
    player::PlayerComponents,
    runtime::Runtime,
    server::Server,
    values::{Vec3, id},
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
    event.player.set_gamemode(Gamemode::Survival)?;
    event
        .player
        .inventory()?
        .set_slot(36, ItemStack::new(id![minecraft:diamond_pickaxe]))?;

    event
        .player
        .inventory()?
        .set_slot(37, ItemStack::new(id![minecraft:diamond_shovel]))?;

    event.player.inventory()?.set_slot(
        38,
        ItemStack::new(id![minecraft:cobblestone]).with(ItemComponents::ITEM_COUNT, 64),
    )?;

    Runtime::spawn_task(move || {
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

fn on_break(event: Arc<BreakBlockEvent>) -> ActorResult<()> {
    let dim = event.player.dimension()?;
    dim.set_block(event.position, event.old_block.clone())?;
    Ok(())
}

fn on_place(event: Arc<PlaceBlockEvent>) -> ActorResult<()> {
    let dim = event.player.dimension()?;
    dim.set_block(event.position, BlockState::new(id![minecraft:air]))?;
    Ok(())
}
