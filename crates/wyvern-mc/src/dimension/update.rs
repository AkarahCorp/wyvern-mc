use voxidian_protocol::packet::s2c::play::EntityPositionSyncS2CPlayPacket;

use crate::{actors::ActorResult, dimension::DimensionData, runtime::Runtime, server::Server};

use super::EntityComponents;

impl DimensionData {
    pub fn propogate_entities(&mut self) -> ActorResult<()> {
        let players = self.players()?.clone();
        for entity in &mut self.entities {
            let pos = entity.1.components.get(EntityComponents::POSITION)?;
            let dir = entity.1.components.get(EntityComponents::DIRECTION)?;
            let id = entity.1.components.get(EntityComponents::ENTITY_ID)?;
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
        Ok(())
    }
}
