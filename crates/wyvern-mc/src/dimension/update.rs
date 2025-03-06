use voxidian_protocol::packet::s2c::play::EntityPositionSyncS2CPlayPacket;

use crate::{
    actors::ActorResult,
    components::{DataComponentHolder, DataComponentPatch},
    dimension::DimensionData,
    runtime::Runtime,
    server::Server,
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
        for entity in self.entities.iter_mut().map(|x| x.1) {
            log::error!("e: {:?}", entity.get(EntityComponents::UUID));
            if let Ok(true) = entity.components.get(EntityComponents::PHYSICS_ENABLED) {
                if let Ok(velocity) = entity.components.get(EntityComponents::VELOCITY) {
                    let pos = entity.get(EntityComponents::POSITION)?;
                    entity.set(
                        EntityComponents::POSITION,
                        pos.with_x(pos.x() + velocity.x())
                            .with_y(pos.y() + velocity.y())
                            .with_z(pos.z() + velocity.z()),
                    );
                }
            }
        }
        Ok(())
    }
}
