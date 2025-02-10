use std::{collections::HashMap, fmt::Debug};

use blocks::BlockState;
use chunk::{Chunk, ChunkSection};
use entity::{Entity, EntityData, EntityType};
use tokio::sync::mpsc::{Sender, channel};
use voxidian_protocol::{
    packet::s2c::play::{AddEntityS2CPlayPacket, BlockUpdateS2CPlayPacket},
    registry::RegEntry,
    value::{Angle, BlockPos, DimType, EntityMetadata, EntityType as PtcEntityType, Uuid, VarInt},
};

use crate::{
    events::ChunkLoadEvent,
    server::Server,
    values::{Key, Vec2, Vec3},
};

pub mod blocks;
pub mod chunk;
pub mod entity;
pub mod properties;

#[allow(dead_code)]
#[crate::actor(Dimension, DimensionMessage)]
pub struct DimensionData {
    #[allow(unused)]
    pub(crate) name: Key<DimensionData>,
    pub(crate) chunks: HashMap<Vec2<i32>, Chunk>,
    pub(crate) entities: HashMap<Uuid, EntityData>,
    pub(crate) server: Option<Server>,
    pub(crate) sender: Sender<DimensionMessage>,
    pub(crate) dim_type: Key<DimType>,
    pub(crate) chunk_generator: fn(&mut Chunk, i32, i32),
}

#[crate::message(Dimension, DimensionMessage)]
impl DimensionData {
    #[GetName]
    pub async fn get_name(&self) -> Key<Dimension> {
        self.name.clone().retype()
    }

    #[GetServer]
    pub async fn get_server(&self) -> Option<Server> {
        self.server.clone()
    }

    #[GetChunkSection]
    pub async fn get_chunk_section(&mut self, position: Vec3<i32>) -> ChunkSection {
        let chunk_pos = Vec2::new(position.x(), position.z());
        self.try_initialize_chunk(&chunk_pos).await;

        let chunk = self.chunks.get_mut(&chunk_pos).unwrap();
        let chunk_y = position.y() / 16;
        chunk.section_at_mut(chunk_y).unwrap().clone()
    }

    #[SetBlock]
    pub async fn set_block(&mut self, position: Vec3<i32>, block_state: BlockState) {
        let chunk_pos = Vec2::new(position.x() / 16, position.z() / 16);
        let pos_in_chunk = Vec3::new(position.x() % 16, position.y(), position.z() % 16);

        self.try_initialize_chunk(&chunk_pos).await;

        let chunk = self.chunks.get_mut(&chunk_pos).unwrap();
        chunk.set_block_at(pos_in_chunk, block_state.clone());

        let server = self.server.clone().unwrap();
        tokio::spawn(async move {
            for conn in server.connections().await {
                let block_state = block_state.clone();
                let pos = position;
                let conn = conn.clone();
                tokio::spawn(async move {
                    if conn.is_loaded_in_world().await {
                        conn.write_packet(BlockUpdateS2CPlayPacket {
                            pos: BlockPos::new(pos.x(), pos.y(), pos.z()),
                            block: unsafe {
                                RegEntry::new_unchecked(block_state.protocol_id() as usize)
                            },
                        })
                        .await;
                    }
                });
            }
        });
    }

    #[GetBlock]
    pub async fn get_block_at(&mut self, position: Vec3<i32>) -> BlockState {
        let chunk = Vec2::new(position.x() / 16, position.z() / 16);
        let pos_in_chunk = Vec3::new(position.x() % 16, position.y(), position.z() % 16);

        self.try_initialize_chunk(&chunk).await;

        let chunk = self.chunks.get_mut(&chunk).unwrap();
        chunk.get_block_at(pos_in_chunk)
    }

    #[GetDimType]
    pub async fn get_dimension_type(&mut self) -> Key<DimType> {
        self.dim_type.clone()
    }

    #[SetChunkGenerator]
    pub async fn set_chunk_generator(&mut self, function: fn(&mut Chunk, i32, i32)) {
        self.chunk_generator = function;
    }

    #[GetAllEntities]
    pub async fn get_all_entities(&self) -> Vec<Entity> {
        self.entities
            .values()
            .map(|x| Entity {
                dimension: Dimension {
                    sender: self.sender.clone(),
                },
                uuid: x.uuid,
            })
            .collect()
    }

    #[SpawnEntity]
    pub async fn spawn_entity(&mut self, entity_type: Key<EntityType>) -> Entity {
        let mut uuid = Uuid::new_v4();
        while self.entities.contains_key(&uuid) {
            uuid = Uuid::new_v4();
        }

        let id = self.server.clone().unwrap().get_entity_id().await;

        self.entities.insert(uuid, EntityData {
            entity_type: entity_type.clone(),
            uuid,
            id,
            position: Vec3::new(0.0, 0.0, 0.0),
            heading: Vec2::new(0.0, 0.0),
            metadata: EntityMetadata::new(),
        });

        for conn in self.server.clone().unwrap().connections().await {
            if let Some(dim) = conn.get_dimension().await {
                if dim.sender.same_channel(&self.sender) {
                    conn.write_packet(AddEntityS2CPlayPacket {
                        id: id.into(),
                        uuid,
                        kind: PtcEntityType::vanilla_registry()
                            .make_entry(&entity_type.clone().into())
                            .unwrap(),
                        x: 0.0,
                        y: 0.0,
                        z: 0.0,
                        pitch: Angle::of_deg(0.0),
                        yaw: Angle::of_deg(0.0),
                        head_yaw: Angle::of_deg(0.0),
                        data: VarInt::from(0),
                        vel_x: 0,
                        vel_y: 0,
                        vel_z: 0,
                    })
                    .await;
                }
            };
        }

        Entity {
            dimension: Dimension {
                sender: self.sender.clone(),
            },
            uuid,
        }
    }

    #[ManipulateEntity]
    pub async fn manipulate_entity(
        &mut self,
        uuid: Uuid,
        f: Box<dyn Fn(&mut EntityData) + Send + Sync>,
    ) {
        if let Some(entity) = self.entities.get_mut(&uuid) {
            f(entity);
        }
    }

    #[ReadEntity]
    pub async fn read_entity(&self, uuid: Uuid, f: Box<dyn Fn(&EntityData) + Send + Sync>) {
        if let Some(entity) = self.entities.get(&uuid) {
            f(entity);
        }
    }
}

impl DimensionData {
    pub(crate) fn new(
        name: Key<DimensionData>,
        server: Server,
        dim_type: Key<DimType>,
    ) -> DimensionData {
        let chan = channel(1024);
        DimensionData {
            name,
            chunks: HashMap::new(),
            entities: HashMap::new(),
            server: Some(server),
            receiver: chan.1,
            sender: chan.0,
            dim_type,
            chunk_generator: |_, _, _| {},
        }
    }

    pub(crate) async fn try_initialize_chunk(&mut self, pos: &Vec2<i32>) {
        if !self.chunks.contains_key(pos) {
            let server = self.server.clone().unwrap();
            let registries = server.registries().await;

            let dim_type = registries
                .dimension_types
                .get(&self.dim_type.clone().into())
                .unwrap();

            let min_sections = dim_type.min_y / 16;
            let max_sections = dim_type.max_y / 16;

            let mut chunk = Chunk::new(min_sections, max_sections);
            (self.chunk_generator)(&mut chunk, pos.x(), pos.y());
            self.chunks.insert(*pos, chunk);

            server.spawn_event(ChunkLoadEvent {
                dimension: Dimension {
                    sender: self.sender.clone(),
                },
                pos: *pos,
            });
        }
    }
}
