use std::sync::{
    Arc, Mutex,
    atomic::{AtomicI32, Ordering},
};

use voxidian_protocol::{
    packet::s2c::play::EntityPositionSyncS2CPlayPacket,
    value::{EntityMetadata, Uuid},
};

use crate::{
    runtime::Runtime,
    values::{Key, Vec2, Vec3},
};

use super::Dimension;

#[derive(Debug)]
pub struct Entity {
    pub(crate) dimension: Dimension,
    pub(crate) uuid: Uuid,
}

impl Entity {
    pub fn uuid(&self) -> &Uuid {
        &self.uuid
    }

    pub fn dimension(&self) -> &Dimension {
        &self.dimension
    }

    pub async fn remove(&self) {
        self.dimension.remove_entity(self.uuid).await;
    }

    pub async fn entity_id(&self) -> i32 {
        let id = Arc::new(AtomicI32::new(0));
        let id_clone = id.clone();
        self.dimension
            .read_entity(
                self.uuid,
                Box::new(move |entity| {
                    id_clone.store(entity.id, Ordering::Relaxed);
                }),
            )
            .await;
        id.load(Ordering::Relaxed)
    }

    pub async fn entity_type(&self) -> Key<EntityType> {
        let value = Arc::new(Mutex::new(Key::new("?", "?")));
        let value_clone = value.clone();
        self.dimension
            .read_entity(
                *self.uuid(),
                Box::new(move |entity| {
                    *value_clone.lock().unwrap() = entity.entity_type.clone();
                }),
            )
            .await;
        value.lock().unwrap().clone()
    }

    pub async fn position(&self) -> (Vec3<f64>, Vec2<f32>) {
        let value = Arc::new(Mutex::new((Vec3::new(0.0, 0.0, 0.0), Vec2::new(0.0, 0.0))));
        let value_clone = value.clone();
        self.dimension
            .read_entity(
                *self.uuid(),
                Box::new(move |entity| {
                    *value_clone.lock().unwrap() = (entity.position, entity.heading);
                }),
            )
            .await;
        *value.lock().unwrap()
    }

    pub async fn teleport(&mut self, position: Vec3<f64>) {
        let dimension = self.dimension.clone();
        let server = self.dimension.get_server().await.unwrap();

        self.dimension
            .manipulate_entity(
                self.uuid,
                Box::new(move |entity: &mut EntityData| {
                    entity.position = position;

                    let entity = Arc::new(entity.clone());

                    let server = server.clone();
                    let dimension = dimension.clone();
                    Runtime::spawn(async move {
                        let dimension = dimension.clone();
                        for conn in server.connections().await {
                            let Some(dim) = conn.get_dimension().await else {
                                continue;
                            };
                            if !dim.sender.same_channel(&dimension.sender) {
                                continue;
                            }
                            conn.write_packet(EntityPositionSyncS2CPlayPacket {
                                entity_id: entity.id.into(),
                                x: entity.position.x(),
                                y: entity.position.y(),
                                z: entity.position.z(),
                                vx: 0.0,
                                vy: 0.0,
                                vz: 0.0,
                                yaw: entity.heading.x(),
                                pitch: entity.heading.y(),
                                on_ground: false,
                            })
                            .await;
                        }
                    });
                }),
            )
            .await;
    }

    pub async fn rotate(&mut self, heading: Vec2<f32>) {
        let dimension = self.dimension.clone();
        let server = self.dimension.get_server().await.unwrap();

        self.dimension
            .manipulate_entity(
                self.uuid,
                Box::new(move |entity: &mut EntityData| {
                    entity.heading = heading;

                    let entity = Arc::new(entity.clone());

                    let server = server.clone();
                    let dimension = dimension.clone();
                    Runtime::spawn(async move {
                        let dimension = dimension.clone();
                        for conn in server.connections().await {
                            let Some(dim) = conn.get_dimension().await else {
                                continue;
                            };
                            if !dim.sender.same_channel(&dimension.sender) {
                                continue;
                            }
                            conn.write_packet(EntityPositionSyncS2CPlayPacket {
                                entity_id: entity.id.into(),
                                x: entity.position.x(),
                                y: entity.position.y(),
                                z: entity.position.z(),
                                vx: 0.0,
                                vy: 0.0,
                                vz: 0.0,
                                yaw: entity.heading.x(),
                                pitch: entity.heading.y(),
                                on_ground: false,
                            })
                            .await;
                        }
                    });
                }),
            )
            .await;
    }
}

pub struct EntityType;

#[allow(unused)]
#[derive(Debug, Clone)]
pub struct EntityData {
    pub(crate) entity_type: Key<EntityType>,
    pub(crate) uuid: Uuid,
    pub(crate) id: i32,

    pub(crate) position: Vec3<f64>,
    pub(crate) heading: Vec2<f32>,

    pub(crate) metadata: EntityMetadata,
}

pub struct Entities;
wyvern_macros::generate_entity_types!();
