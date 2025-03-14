use std::sync::Arc;

use wyvern_mc::{
    actors::ActorResult,
    blocks::{BlockState, Blocks},
    events::{
        DimensionCreateEvent, PlayerCommandEvent, PlayerJoinEvent, ServerStartEvent,
        ServerTickEvent,
    },
    player::PlayerComponents,
    server::Server,
    values::{Vec3, id, regval::DimensionType},
};

fn main() {
    env_logger::init();

    Server::builder()
        .event(on_server_start)
        .event(on_dim_init)
        .event(on_join)
        .event(tick)
        .event(on_command)
        .registries(|registries| {
            registries.add_defaults();
            registries.dimension_type(
                id![minecraft:overworld],
                DimensionType::default().height(128).min_y(-64),
            );
        })
        .run();
}

fn on_server_start(event: Arc<ServerStartEvent>) -> ActorResult<()> {
    event.server.create_dimension(id!(example:root))?;

    Ok(())
}

fn tick(event: Arc<ServerTickEvent>) -> ActorResult<()> {
    for player in event.server.players()? {
        if player.get(PlayerComponents::POSITION)?.y() < -64.0 {
            player.set(
                PlayerComponents::TELEPORT_POSITION,
                Vec3::new(0.0, 11.0, 0.0),
            )?;
        }
    }
    Ok(())
}

fn on_command(event: Arc<PlayerCommandEvent>) -> ActorResult<()> {
    if event.command == "restart" {
        event.player.set(
            PlayerComponents::TELEPORT_POSITION,
            Vec3::new(0.0, 11.0, 0.0),
        )?;
    }
    Ok(())
}

fn on_dim_init(event: Arc<DimensionCreateEvent>) -> ActorResult<()> {
    let mut block_pos = Vec3::new(0, 10, 0);
    for _ in 0..20 {
        event
            .dimension
            .set_block(block_pos, BlockState::new(Blocks::POLISHED_ANDESITE))?;

        match rand::random_range(1..=3) {
            1 => {
                let ys = rand::random_range(-1..=1);
                block_pos = block_pos
                    .with_x(block_pos.x() + 4 - ys)
                    .with_y(block_pos.y() + ys)
                    .with_z(block_pos.z() + rand::random_range(-2..=2));
            }
            2 => {
                block_pos = block_pos
                    .with_x(block_pos.x() + 6)
                    .with_y(block_pos.y() - 5)
                    .with_z(block_pos.z() + rand::random_range(-2..=2));
                event
                    .dimension
                    .set_block(block_pos, BlockState::new(Blocks::SLIME_BLOCK))?;

                let ys = rand::random_range(-1..=1);
                block_pos = block_pos
                    .with_x(block_pos.x() + 4 - ys)
                    .with_y(block_pos.y() + ys + 2)
                    .with_z(block_pos.z() + rand::random_range(-2..=2));
            }
            3 => {
                block_pos = block_pos.with_x(block_pos.x() + 1);
                event
                    .dimension
                    .set_block(block_pos, BlockState::new(Blocks::POLISHED_ANDESITE))?;
                block_pos = block_pos.with_x(block_pos.x() + 1);
                event
                    .dimension
                    .set_block(block_pos, BlockState::new(Blocks::POLISHED_ANDESITE))?;
                block_pos = block_pos
                    .with_x(block_pos.x() + 1)
                    .with_y(block_pos.y() + 2);
                event
                    .dimension
                    .set_block(block_pos, BlockState::new(Blocks::POLISHED_ANDESITE))?;
                block_pos = block_pos
                    .with_x(block_pos.x() + 1)
                    .with_y(block_pos.y() - 2);
                event
                    .dimension
                    .set_block(block_pos, BlockState::new(Blocks::POLISHED_ANDESITE))?;
                block_pos = block_pos.with_x(block_pos.x() + 1);
                event
                    .dimension
                    .set_block(block_pos, BlockState::new(Blocks::POLISHED_ANDESITE))?;
            }
            _ => unreachable!(),
        }
    }
    Ok(())
}

fn on_join(event: Arc<PlayerJoinEvent>) -> ActorResult<()> {
    event.new_dimension.set(id![example:root]);
    event.player.set(
        PlayerComponents::TELEPORT_POSITION,
        Vec3::new(0.0, 11.0, 0.0),
    )?;
    Ok(())
}
