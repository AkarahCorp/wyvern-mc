use std::sync::Arc;

use wyvern_mc::{
    actors::ActorResult,
    blocks::{BlockComponents, BlockState},
    components::DataComponentHolder,
    entities::{AttributeContainer, Attributes, EntityComponents},
    events::{DimensionCreateEvent, PlayerAttackEntityEvent, PlayerJoinEvent, ServerStartEvent},
    id,
    inventory::Inventory,
    item::{ItemComponents, ItemStack},
    runtime::Runtime,
    server::Server,
    values::{Sounds, Texts, Vec3},
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
    for x in -20..20 {
        for z in -20..20 {
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
    event.player.teleport(Vec3::new(0.0, 1.0, 0.0))?;

    event.player.set_attributes(
        AttributeContainer::new()
            .with(Attributes::MAX_HEALTH, 30.0)
            .with(Attributes::ATTACK_SPEED, 900.0)
            .with(Attributes::FOLLOW_RANGE, 0.0),
    )?;
    Runtime::spawn_task(move || {
        let entity = event
            .player
            .dimension()?
            .spawn_entity(id![minecraft:zombie])?;
        entity.set(EntityComponents::POSITION, Vec3::new(3.0, 1.0, 3.0))?;
        entity.set(EntityComponents::PHYSICS_ENABLED, false)?;
        Ok(())
    });

    Ok(())
}

fn on_attack(event: Arc<PlayerAttackEntityEvent>) -> ActorResult<()> {
    event
        .player
        .send_message(Texts::literal("HI YOU HIT AN ENTITY WOW"))?;

    event.player.inventory()?.set_slot(
        36,
        ItemStack::new(id![minecraft:diamond_sword])
            .with(
                ItemComponents::ITEM_NAME,
                Texts::literal("Diamond Sword").into(),
            )
            .with(ItemComponents::ITEM_MODEL, id![minecraft:diamond_sword]),
    )?;

    let dir = event.player.direction()?.to_3d_direction().map(|x| x / 2.0);

    event
        .entity
        .set(EntityComponents::VELOCITY, dir.with_y(0.3))?;

    event.entity.set(EntityComponents::GRAVITY_ENABLED, true)?;
    event.entity.set(EntityComponents::PHYSICS_ENABLED, true)?;

    event.player.play_sound(Sounds::ENTITY_PLAYER_ATTACK_CRIT)?;

    Ok(())
}
