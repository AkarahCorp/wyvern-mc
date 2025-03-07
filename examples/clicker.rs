use std::{
    collections::HashMap,
    sync::{Arc, LazyLock, Mutex},
};

use voxidian_protocol::packet::s2c::play::ScreenWindowKind;
use wyvern_mc::{
    actors::ActorResult,
    blocks::BlockState,
    components::DataComponentHolder,
    entities::{Entities, EntityComponents},
    events::{
        DimensionCreateEvent, PlayerJoinEvent, RightClickEvent, ServerStartEvent, ServerTickEvent,
        SwapHandsEvent,
    },
    id,
    inventory::Inventory,
    item::{ItemComponents, ItemStack},
    player::PlayerComponents,
    server::Server,
    values::{Id, SoundCategory, Sounds, Texts, Uuid, Vec2, Vec3, nbt::NbtCompound},
};

static COUNTER: LazyLock<Mutex<HashMap<Uuid, i32>>> = LazyLock::new(|| Mutex::new(HashMap::new()));

fn main() {
    env_logger::init();

    Server::builder()
        .event(on_server_start)
        .event(on_dim_init)
        .event(on_join)
        .event(on_tick)
        .event(on_right_click)
        .event(on_swap_hands)
        .registries(|registries| {
            registries.add_defaults();
        })
        .run();
}

fn on_server_start(event: Arc<ServerStartEvent>) -> ActorResult<()> {
    event.server.create_dimension(id!(clicker:root))?;

    Ok(())
}

fn on_dim_init(event: Arc<DimensionCreateEvent>) -> ActorResult<()> {
    for x in 0..6 {
        for z in 0..6 {
            event.dimension.set_block(
                Vec3::new(x, 0, z),
                BlockState::new(id![minecraft:grass_block]),
            )?;
        }
    }

    let entity = event.dimension.spawn_entity(Entities::ZOMBIE)?;
    entity.set(EntityComponents::POSITION, Vec3::new(1.0, 0.0, 2.0))?;
    entity.set(EntityComponents::DIRECTION, Vec2::new(58.0, 32.5))?;
    Ok(())
}

fn on_join(event: Arc<PlayerJoinEvent>) -> ActorResult<()> {
    event.new_dimension.set(id!(clicker:root));

    event.player.inventory()?.set_slot(
        40,
        ItemStack::new(Id::new("minecraft", "diamond"))
            .with(ItemComponents::CUSTOM_DATA, {
                let mut compound = NbtCompound::new();
                compound.set("clicker_data", 10.into());
                compound
            })
            .with(
                ItemComponents::ITEM_NAME,
                Texts::literal("Click me!").into(),
            ),
    )?;
    Ok(())
}

fn on_tick(event: Arc<ServerTickEvent>) -> ActorResult<()> {
    for player in event.server.players()? {
        let uuid = player.get(PlayerComponents::UUID)?;

        let count = {
            let mut counter = COUNTER.lock().unwrap();

            match counter.get(&uuid).copied() {
                None => {
                    counter.insert(uuid, 0);
                    0
                }
                Some(count) => count,
            }
        };

        player.send_action_bar(Texts::literal(format!("Clicks: {:?}", count)))?;

        for entity in player.dimension()?.entities()? {
            entity.set(
                EntityComponents::POSITION,
                Vec3::new(
                    rand::random_range(0.0..1.0),
                    rand::random_range(0.0..1.0),
                    rand::random_range(0.0..1.0),
                ),
            )?;
            entity.set(
                EntityComponents::DIRECTION,
                Vec2::new(
                    rand::random_range(-90.0..90.0),
                    rand::random_range(-180.0..180.0),
                ),
            )?;
        }
    }
    Ok(())
}

fn on_right_click(event: Arc<RightClickEvent>) -> ActorResult<()> {
    let uuid = event.player.get(PlayerComponents::UUID)?;
    {
        let mut counter = COUNTER.lock().unwrap();
        if let Some(number) = counter.get_mut(&uuid) {
            *number += 1;
        };
    }

    event.player.play_sound(
        Sounds::BLOCK_AMETHYST_CLUSTER_BREAK
            .pitch(1.5)
            .volume(0.7)
            .category(SoundCategory::Master),
    )?;

    let item = event.player.inventory()?.get_slot(40)?;
    log::error!("{:?}", item.get(ItemComponents::CUSTOM_DATA));
    log::error!("{:?}", item.get(ItemComponents::ITEM_NAME));

    log::error!("{:?}", item.component_map());
    Ok(())
}

fn on_swap_hands(event: Arc<SwapHandsEvent>) -> ActorResult<()> {
    event.player.open_screen(ScreenWindowKind::Generic9x1)?;
    Ok(())
}
