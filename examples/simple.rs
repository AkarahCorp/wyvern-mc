use std::sync::{Arc, LazyLock};

use noise::{NoiseFn, Simplex};
use voxidian_protocol::packet::s2c::play::ScreenWindowKind;
use wyvern_mc::{
    actors::ActorResult,
    blocks::{BlockComponents, BlockState, Blocks},
    components::DataComponentHolder,
    dimension::chunk::Chunk,
    entities::EntityComponents,
    events::{
        BreakBlockEvent, ChatMessageEvent, DimensionCreateEvent, DropItemEvent, PlaceBlockEvent,
        PlayerCommandEvent, PlayerJoinEvent, ServerStartEvent, ServerTickEvent,
    },
    id,
    inventory::Inventory,
    item::{ItemComponents, ItemStack},
    runtime::Runtime,
    server::Server,
    values::{Id, Text, TextColor, Texts, Vec3, regval::DimensionType},
};

fn main() {
    env_logger::init();

    Server::builder()
        .event(on_server_start)
        .event(on_server_tick)
        .event(dim_init)
        .event(on_command)
        .event(on_drop_item)
        .event(on_place)
        .event(on_break)
        .event(on_chat)
        .event(on_join)
        .registries(|registries| {
            registries.add_defaults();
            registries.dimension_type(
                Id::new("minecraft", "overworld"),
                DimensionType::default().min_y(-32).height(64),
            );
        })
        .run();
}

static SIMPLEX: LazyLock<Simplex> = LazyLock::new(|| Simplex::new(0));

fn on_command(event: Arc<PlayerCommandEvent>) -> ActorResult<()> {
    if event.command.as_str() == "overload" {
        let event = event.clone();
        Runtime::spawn_task(move || {
            let state = BlockState::new(Id::new("minecraft", "grass_block"))
                .with(BlockComponents::SNOWY, false);
            let dim = event.player.dimension().unwrap();
            for x in 1..100 {
                for y in 1..10 {
                    for z in 1..100 {
                        dim.set_block(Vec3::new(x, y, z), state.clone())?;
                    }
                }
            }
            Ok(())
        });
    }

    if event.command == "rootdir" {
        let dimension = event.player.server()?.dimension(id!(wyvern:root))?;
        event.player.set_dimension(dimension)?;
    }

    if event.command == "altdir" {
        let dimension = event.player.server()?.dimension(id!(example:alternate))?;
        event.player.set_dimension(dimension)?;
    }

    if event.command == "openscreen" {
        event.player.open_screen(ScreenWindowKind::Generic9x5)?;

        event
            .player
            .set_screen_slot(0, ItemStack::new(id![minecraft:diamond]))?;
        event
            .player
            .set_screen_slot(1, ItemStack::new(id![minecraft:iron_block]))?;
    }

    Ok(())
}

fn dim_init(event: Arc<DimensionCreateEvent>) -> ActorResult<()> {
    event
        .dimension
        .set_chunk_generator(move |chunk: &mut Chunk, x, z| {
            for x2 in 0..16 {
                for z2 in 0..16 {
                    let y = SIMPLEX.get([
                        (x2 + (x * 16)) as f64 / 100.0,
                        (z2 + (z * 16)) as f64 / 100.0,
                    ]) + 1.0;
                    let y = f64::floor(y * -16.0 + 8.0) as i32;

                    let new_pos = Vec3::new(x2, y, z2);
                    chunk.set_block_at(new_pos, BlockState::new(Blocks::GRASS_BLOCK));

                    for y in -32..y {
                        let new_pos = Vec3::new(x2, y, z2);
                        chunk.set_block_at(new_pos, BlockState::new(Blocks::DIRT));
                    }
                }
            }
        })?;

    event.dimension.max_chunks(3, 3)?;

    Ok(())
}

fn on_server_tick(event: Arc<ServerTickEvent>) -> ActorResult<()> {
    for dim in event.server.dimensions()? {
        log::debug!("Dim players: {:?}", dim.players());
        for mut entity in dim.entities()? {
            let new_pos = Vec3::new(
                rand::random::<f64>() * 128.0,
                rand::random::<f64>() * 16.0,
                rand::random::<f64>() * 128.0,
            );
            entity.set(EntityComponents::POSITION, new_pos)?;
        }
    }

    for player in event.server.players()? {
        player.inventory()?.set_slot(
            38,
            ItemStack::new(Id::new("minecraft", "netherite_axe"))
                .with(ItemComponents::MAX_DAMAGE, 1500)
                .with(ItemComponents::DAMAGE, 1)
                .with(
                    ItemComponents::ITEM_NAME,
                    Texts::literal("Normal Netherite Axe")
                        .with_color(TextColor::new(0, 255, 0))
                        .into(),
                )
                .with(
                    ItemComponents::LORE,
                    vec![
                        Texts::literal("Fancy lore line.")
                            .with_color(TextColor::new(133, 133, 133))
                            .into(),
                        Texts::literal("How cool!")
                            .with_color(TextColor::new(182, 18, 238))
                            .into(),
                    ],
                ),
        )?;
    }

    Ok(())
}

fn on_server_start(event: Arc<ServerStartEvent>) -> ActorResult<()> {
    event.server.create_dimension(id!(example:root))?;
    event.server.create_dimension(id!(example:alternate))?;

    Ok(())
}

fn on_drop_item(event: Arc<DropItemEvent>) -> ActorResult<()> {
    event
        .player
        .send_message(Texts::literal("You dropped an item, wow!"))?;

    Ok(())
}

fn on_place(event: Arc<PlaceBlockEvent>) -> ActorResult<()> {
    event
        .player
        .send_message(Texts::literal("You placed an item, wow!"))?;
    Ok(())
}

fn on_break(event: Arc<BreakBlockEvent>) -> ActorResult<()> {
    event
        .player
        .send_message(Texts::literal("You broke an item, wow!"))?;
    Ok(())
}

fn on_chat(event: Arc<ChatMessageEvent>) -> ActorResult<()> {
    for player in Server::get()?.players()? {
        player.send_message(
            Texts::literal(format!("{}: {}", event.player.username()?, event.message))
                .with_color(TextColor::new(0, 255, 0)),
        )?;
    }
    Ok(())
}

fn on_join(event: Arc<PlayerJoinEvent>) -> ActorResult<()> {
    event.new_dimension.set(id!(example:root));

    event.player.inventory()?.set_slot(
        36,
        ItemStack::new(Id::new("minecraft", "stone"))
            .with(ItemComponents::MAX_DAMAGE, 10)
            .with(ItemComponents::DAMAGE, 1)
            .with(
                ItemComponents::ITEM_MODEL,
                Id::constant("minecraft", "stone"),
            ),
    )?;

    event.player.inventory()?.set_slot(
        37,
        ItemStack::new(Id::new("minecraft", "diamond_sword"))
            .with(ItemComponents::MAX_DAMAGE, 20)
            .with(ItemComponents::DAMAGE, 6)
            .with(
                ItemComponents::ITEM_MODEL,
                Id::constant("minecraft", "diamond_sword"),
            ),
    )?;
    Ok(())
}
