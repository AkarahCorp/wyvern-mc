use std::sync::Arc;

use wyvern_mc::{
    actors::ActorResult,
    blocks::{BlockState, Blocks},
    datatypes::regval::DimensionType,
    events::{
        DimensionCreateEvent, PlayerCommandEvent, PlayerJoinEvent, PlayerRespawnEvent,
        ServerStartEvent, ServerTickEvent,
    },
    player::{HealthComponent, PlayerComponents},
    server::{Server, registries::RegistryKeys},
    values::{DVec3, IVec3, Id, id},
};

fn main() {
    env_logger::init();

    Server::builder()
        .event(on_server_start)
        .event(on_dim_init)
        .event(on_join)
        .event(tick)
        .event(on_command)
        .event(on_respawn)
        .registries(|registries| {
            registries.get_mut(RegistryKeys::DIMENSION_TYPE).insert(
                Id::new("minecraft", "overworld"),
                DimensionType::default().min_y(-64).height(128),
            );
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

async fn tick(event: Arc<ServerTickEvent>) -> ActorResult<()> {
    for player in event.server.players()? {
        if player.get(PlayerComponents::POSITION)?[1] < -64.0 {
            player.set(
                PlayerComponents::TELEPORT_POSITION,
                DVec3::new(0.0, 1000.0, 0.0),
            )?;
            player.set(
                PlayerComponents::HEALTH,
                HealthComponent {
                    health: 0.0,
                    food: 20,
                    saturation: 20.0,
                },
            )?;
        }
    }
    Ok(())
}

async fn on_command(event: Arc<PlayerCommandEvent>) -> ActorResult<()> {
    if event.command == "restart" {
        event.player.set(
            PlayerComponents::TELEPORT_POSITION,
            DVec3::new(0.0, 11.0, 0.0),
        )?;
    }
    Ok(())
}

async fn on_dim_init(event: Arc<DimensionCreateEvent>) -> ActorResult<()> {
    let mut block_pos = IVec3::new(0, 10, 0);
    for _ in 0..20 {
        event
            .dimension
            .set_block(block_pos, BlockState::new(Blocks::POLISHED_ANDESITE))?;

        match rand::random_range(1..=3) {
            1 => {
                let ys = rand::random_range(-1..=1);
                block_pos += IVec3::new(4 - ys, ys, rand::random_range(-2..=2));
            }
            2 => {
                block_pos += IVec3::new(6, -5, rand::random_range(-2..=2));

                event
                    .dimension
                    .set_block(block_pos, BlockState::new(Blocks::SLIME_BLOCK))?;

                let ys = rand::random_range(-1..=1);
                block_pos += IVec3::new(4 - ys, ys + 2, rand::random_range(-2..=2));
            }
            3 => {
                block_pos += IVec3::new(1, 0, 0);
                event
                    .dimension
                    .set_block(block_pos, BlockState::new(Blocks::POLISHED_ANDESITE))?;
                block_pos += IVec3::new(1, 0, 0);
                event
                    .dimension
                    .set_block(block_pos, BlockState::new(Blocks::POLISHED_ANDESITE))?;
                block_pos += IVec3::new(2, 2, 0);
                event
                    .dimension
                    .set_block(block_pos, BlockState::new(Blocks::POLISHED_ANDESITE))?;
                block_pos += IVec3::new(1, -2, 0);
                event
                    .dimension
                    .set_block(block_pos, BlockState::new(Blocks::POLISHED_ANDESITE))?;
                block_pos = block_pos.with_x(block_pos[0] + 1);
                event
                    .dimension
                    .set_block(block_pos, BlockState::new(Blocks::POLISHED_ANDESITE))?;
            }
            _ => unreachable!(),
        }
    }
    Ok(())
}

async fn on_join(event: Arc<PlayerJoinEvent>) -> ActorResult<()> {
    event.new_dimension.set(id![example:root]);
    event.player.set(
        PlayerComponents::TELEPORT_POSITION,
        DVec3::new(0.0, 11.0, 0.0),
    )?;
    Ok(())
}

async fn on_respawn(event: Arc<PlayerRespawnEvent>) -> ActorResult<()> {
    event.player.set(
        PlayerComponents::TELEPORT_POSITION,
        DVec3::new(0.0, 100.0, 0.0),
    )?;
    Ok(())
}
