use voxidian_protocol::packet::s2c::play::EntityPositionSyncS2CPlayPacket;

use crate::{
    actors::ActorResult,
    blocks::Blocks,
    components::DataComponentPatch,
    dimension::{Dimension, DimensionData},
    entities::Entity,
    runtime::Runtime,
    server::Server,
    values::Vec3,
};

use super::EntityComponents;

impl DimensionData {
    pub fn propogate_entity_packets(&mut self) -> ActorResult<()> {
        let players = self.players()?.clone();
        for entity in &mut self.entities {
            let patch =
                DataComponentPatch::from_maps(&entity.1.last_components, &entity.1.components);

            let id = entity.1.components.get(EntityComponents::ENTITY_ID)?;
            if patch
                .added_fields()
                .contains_type(&EntityComponents::POSITION)
                || patch
                    .added_fields()
                    .contains_type(&EntityComponents::DIRECTION)
            {
                let pos = entity.1.components.get(EntityComponents::POSITION)?;
                let dir = entity.1.components.get(EntityComponents::DIRECTION)?;
                for player in &players {
                    let player = *player;
                    Runtime::spawn_task(move || {
                        let player = Server::get()?.player(player)?;
                        player.write_packet(EntityPositionSyncS2CPlayPacket {
                            entity_id: id.into(),
                            x: pos.x(),
                            y: pos.y(),
                            z: pos.z(),
                            vx: 0.0,
                            vy: 0.0,
                            vz: 0.0,
                            yaw: dir.x(),
                            pitch: dir.y(),
                            on_ground: true,
                        })?;
                        Ok(())
                    });
                }
            }

            entity.1.last_components = entity.1.components.clone();
        }
        Ok(())
    }

    pub fn auto_apply_entity_properties(&mut self) -> ActorResult<()> {
        for entity in &self.entities {
            let entity = Entity {
                dimension: Dimension {
                    sender: self.sender.clone(),
                },
                uuid: *entity.0,
            };
            let dimension = Dimension {
                sender: self.sender.clone(),
            };
            Runtime::spawn_task(move || {
                if let Ok(true) = entity.get(EntityComponents::PHYSICS_ENABLED) {
                    if let Ok(mut velocity) = entity.get(EntityComponents::VELOCITY) {
                        let mut pos = entity.get(EntityComponents::POSITION)?;
                        for _ in 1..10 {
                            let new_pos = pos
                                .with_x(pos.x() + velocity.x())
                                .with_y(pos.y() + velocity.y())
                                .with_z(pos.z() + velocity.z());

                            if dimension
                                .get_block(Vec3::new(
                                    new_pos.x().floor() as i32,
                                    new_pos.y().floor() as i32,
                                    new_pos.z().floor() as i32,
                                ))?
                                .name()
                                == &Blocks::AIR
                            {
                                pos = new_pos;
                                break;
                            } else {
                                velocity = velocity.map(|x| x / 2.0);
                            }
                        }
                        velocity = velocity.map(|x| x * 0.9);
                        entity.set(EntityComponents::POSITION, pos)?;
                        entity.set(EntityComponents::VELOCITY, velocity)?;
                    }

                    if let Ok(true) = entity.get(EntityComponents::GRAVITY_ENABLED) {
                        let vel = entity.get(EntityComponents::VELOCITY)?;
                        entity.set(EntityComponents::VELOCITY, vel.with_y(vel.y() - 0.08))?;
                    }
                }
                Ok(())
            });
        }
        Ok(())
    }
}
