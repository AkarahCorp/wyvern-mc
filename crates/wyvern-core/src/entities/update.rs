use voxidian_protocol::{
    packet::s2c::play::{
        EntityEquipmentPart, EntityPositionSyncS2CPlayPacket, EquipmentSlot,
        RotateHeadS2CPlayPacket, SetEquipmentS2CPlayPacket,
    },
    value::Angle,
};
use wyvern_components::DataComponentPatch;

use crate::{
    actors::ActorResult, blocks::Blocks, dimension::DimensionData, entities::Entity,
    runtime::Runtime, server::Server,
};
use wyvern_values::Vec3;

use super::{Dimension, EntityComponents};

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
                    Runtime::spawn_task(async move {
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
                        player.write_packet(RotateHeadS2CPlayPacket {
                            entity: id.into(),
                            yaw: Angle::of_deg(dir.x().rem_euclid(360.0)),
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
                dimension: {
                    Dimension {
                        sender: self.sender.downgrade(),
                    }
                },
                uuid: *entity.0,
            };
            let dimension = {
                Dimension {
                    sender: self.sender.downgrade(),
                }
            };

            Runtime::spawn_task(async move {
                entity_position(&entity, &dimension)?;
                entity_equipment(&entity)?;
                Ok(())
            });
        }
        Ok(())
    }
}

pub fn entity_position(entity: &Entity, dimension: &Dimension) -> ActorResult<()> {
    if let Ok(true) = entity.get(EntityComponents::PHYSICS_ENABLED) {
        if let Ok(mut velocity) = entity.get(EntityComponents::VELOCITY) {
            let mut pos = entity.get(EntityComponents::POSITION)?;
            for _ in 1..10 {
                let new_pos = pos
                    .with_x(pos.x() + velocity.x())
                    .with_y(pos.y() + velocity.y())
                    .with_z(pos.z() + velocity.z());

                if dimension.get_block(new_pos.floor())?.name() == &Blocks::AIR {
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
    }
    if let Ok(true) = entity.get(EntityComponents::GRAVITY_ENABLED) {
        let vel = entity
            .get(EntityComponents::VELOCITY)
            .unwrap_or(Vec3::new(0.0, 0.0, 0.0));
        entity.set(EntityComponents::VELOCITY, vel.with_y(vel.y() - 0.08))?;
    }
    Ok(())
}

pub fn entity_equipment(entity: &Entity) -> ActorResult<()> {
    let mut parts = Vec::new();
    macro_rules! add_parts {
        (
            $($ty:ident -> $ev:ident),*
        ) => {
            $(if let Ok(item) = entity.get(EntityComponents::$ty) {
                parts.push(EntityEquipmentPart {
                    slot: EquipmentSlot::$ev,
                    item: item.into(),
                });
            };)*
        };
    }

    add_parts!(
        MAINHAND_ITEM -> Mainhand,
        OFFHAND_ITEM -> Offhand,
        BODY_ITEM -> Body,

        HELMET_ITEM -> Helmet,
        CHESTPLATE_ITEM -> Chestplate,
        LEGGINGS_ITEM -> Leggings,
        BOOTS_ITEM -> Boots
    );

    let eid = entity.get(EntityComponents::ENTITY_ID).unwrap_or(-1);

    if !parts.is_empty() {
        for player in entity.dimension().players()? {
            let player = Server::get()?.player(player)?;
            if player.entity_id()? != eid {
                player.write_packet(SetEquipmentS2CPlayPacket {
                    entity_id: eid.into(),
                    parts: parts.clone(),
                })?;
            }
        }
    }
    Ok(())
}
