use std::sync::Arc;

use datafix::serialization::{Codec, DefaultCodec};
use wyvern_mc::{
    actors::ActorResult,
    blocks::{BlockState, Structure},
    components::DataComponentHolder,
    datatypes::{
        nbt::{Nbt, NbtCompound, NbtOps},
        particle::Particle,
        text::{Text, TextColor, Texts},
    },
    entities::{AttributeContainer, Attributes},
    events::{
        BreakBlockEvent, ChatMessageEvent, DimensionCreateEvent, PlaceBlockEvent, PlayerJoinEvent,
        PlayerLeftClickEvent, PlayerLoadEvent, RightClickEvent, ServerStartEvent, ServerTickEvent,
    },
    inventory::Inventory,
    item::{ItemComponents, ItemStack},
    macros::server,
    player::{Player, PlayerComponents},
    server::{Server, ServerBuilder},
    values::{NVec, Vec3, id},
};

#[server]
fn server() -> ServerBuilder {
    Server::builder()
        .event(on_server_start)
        .event(on_dim_init)
        .event(on_join)
        .event(on_break)
        .event(on_place)
        .event(on_shoot)
        .event(on_chat)
        .event(on_tick)
        .event(on_dash)
        .event(on_load)
        .registries(|registries| {
            registries.add_defaults();
        })
}

fn on_server_start(event: Arc<ServerStartEvent>) -> ActorResult<()> {
    event.server.create_dimension(id!(example:root))?;

    Ok(())
}

fn on_dim_init(event: Arc<DimensionCreateEvent>) -> ActorResult<()> {
    let bytes = include_bytes!("./quake.nbt").to_vec();
    let nbt = Nbt::new(NbtCompound::try_from(bytes).unwrap());
    let structure = Structure::codec().decode(&NbtOps, &nbt).unwrap();

    structure.place(event.dimension.clone(), Vec3::new(0, 0, 0))?;
    Ok(())
}

fn on_join(event: Arc<PlayerJoinEvent>) -> ActorResult<()> {
    event.new_dimension.set(id![example:root]);
    event.player.inventory()?.set_slot(
        36,
        ItemStack::new(id![minecraft:iron_hoe])
            .with(ItemComponents::ITEM_NAME, Texts::literal("Railgun").into()),
    )?;
    event.player.set(
        PlayerComponents::ATTRIBUTES,
        AttributeContainer::new().with(Attributes::ATTACK_SPEED, 1000.0),
    )?;
    respawn_player(&event.player)?;
    Ok(())
}

fn on_load(event: Arc<PlayerLoadEvent>) -> ActorResult<()> {
    event.player.send_message(Texts::literal("a"))?;
    respawn_player(&event.player)?;
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

fn on_chat(event: Arc<ChatMessageEvent>) -> ActorResult<()> {
    for player in Server::get()?.players()? {
        player.send_message(Texts::literal(format!(
            "<{}> {}",
            event.player.get(PlayerComponents::USERNAME)?,
            event.message
        )))?;
    }
    Ok(())
}

fn on_tick(_event: Arc<ServerTickEvent>) -> ActorResult<()> {
    for player in Server::get()?.players()? {
        player.set(
            PlayerComponents::SIDEBAR_NAME,
            Texts::literal("QUAKECRAFT")
                .with_color(TextColor::new(255, 255, 0))
                .bold(true)
                .into(),
        )?;
        player.set(
            PlayerComponents::SIDEBAR_LINES,
            vec![
                Texts::literal("").into(),
                Texts::literal("Kills: ")
                    .with_color(TextColor::new(133, 133, 133))
                    .and_then(Texts::literal("Untracked").with_color(TextColor::new(255, 133, 133)))
                    .into(),
                Texts::literal("").into(),
                Texts::literal("www.example.org")
                    .with_color(TextColor::new(255, 255, 0))
                    .into(),
            ],
        )?;
        player.set(PlayerComponents::SIDEBAR_PRESENT, true)?;
    }
    Ok(())
}

fn on_shoot(event: Arc<RightClickEvent>) -> ActorResult<()> {
    let position = event.player.get(PlayerComponents::POSITION)?;
    let direction = event
        .player
        .get(PlayerComponents::DIRECTION)?
        .to_3d_direction()
        .map(|x| x / 2.0);
    let mut step = position.with_y(position.y() + 1.8);
    let players = event.player.dimension()?.players()?;
    for _ in 1..120 {
        step = step
            .with_x(step.x() + direction.x())
            .with_y(step.y() + direction.y())
            .with_z(step.z() + direction.z());

        for subplayer in &players {
            let subplayer = Server::get()?.player(*subplayer)?;
            subplayer.play_particle(step, Particle::new(id![minecraft:electric_spark]))?;
        }

        if *event.player.dimension()?.get_block(step.floor())?.name() != id![minecraft:air] {
            break;
        }

        for player in &players {
            let player = Server::get()?.player(*player)?;
            let position = player.get(PlayerComponents::POSITION)?;
            if player.get(PlayerComponents::USERNAME)
                == event.player.get(PlayerComponents::USERNAME)
            {
                continue;
            }

            let dx = (step.x() - position.x()).abs();
            let dy = (step.y() - (position.y() + 1.0)).abs();
            let dz = (step.z() - position.z()).abs();

            if dx <= 0.5 && dz <= 0.5 && dy <= 1.0 {
                respawn_player(&player)?;

                for subplayer in &players {
                    let subplayer = Server::get()?.player(*subplayer)?;
                    subplayer.send_message(Texts::literal(format!(
                        "{} pommed {}",
                        event.player.get(PlayerComponents::USERNAME)?,
                        player.get(PlayerComponents::USERNAME)?
                    )))?;
                }

                return Ok(());
            }
        }
    }
    Ok(())
}

fn respawn_player(player: &Player) -> ActorResult<()> {
    player.send_message(Texts::literal("a"))?;

    let spawn_pos = loop {
        let rand_x = rand::random_range(40.0..90.0);
        let rand_y = rand::random_range(20.0..40.0);
        let rand_z = rand::random_range(40.0..90.0);

        let mut pos: NVec<f64, 3> = Vec3::new(rand_x, rand_y, rand_z);
        let mut descent_steps = 0;

        while *player.dimension()?.get_block(pos.floor())?.name() == id![minecraft:air] {
            pos = pos.with_y(pos.y() - 0.5);
            descent_steps += 1;
            if descent_steps > 100 {
                break;
            }
        }

        if descent_steps > 10 {
            continue;
        }

        let candidate_spawn = pos.with_y(pos.y() + 1.0);

        if *player
            .dimension()?
            .get_block(candidate_spawn.floor())?
            .name()
            != id![minecraft:air]
        {
            continue;
        }

        let head_space = candidate_spawn.with_y(candidate_spawn.y() + 1.0);
        if *player.dimension()?.get_block(head_space.floor())?.name() != id![minecraft:air] {
            continue;
        }

        break candidate_spawn;
    };

    player.set(PlayerComponents::TELEPORT_POSITION, spawn_pos)?;
    Ok(())
}

fn on_dash(event: Arc<PlayerLeftClickEvent>) -> ActorResult<()> {
    let dir = event
        .player
        .get(PlayerComponents::DIRECTION)?
        .to_3d_direction()
        .with_y(0.1);
    event.player.set(PlayerComponents::TELEPORT_VELOCITY, dir)?;
    Ok(())
}
