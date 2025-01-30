use std::collections::HashMap;

use blocks::BlockState;
use chunk::{Chunk, ChunkSection};
use tokio::sync::mpsc::{Sender, channel};
use voxidian_protocol::{
    packet::s2c::play::BlockUpdateS2CPlayPacket,
    registry::RegEntry,
    value::{BlockPos, DimType},
};

use crate::{
    events::ChunkLoadEvent,
    server::Server,
    values::{Key, Vec2, Vec3},
};

pub mod blocks;
pub mod chunk;
pub mod properties;

#[allow(dead_code)]
#[crate::actor(Dimension, DimensionMessage)]
pub struct DimensionData {
    #[allow(unused)]
    pub(crate) name: Key<DimensionData>,
    pub(crate) chunks: HashMap<Vec2<i32>, Chunk>,
    pub(crate) server: Option<Server>,
    pub(crate) sender: Sender<DimensionMessage>,
    pub(crate) dim_type: Key<DimType>,
    pub(crate) chunk_generator: fn(&mut Chunk, i32, i32),
}

#[crate::message(Dimension, DimensionMessage)]
impl DimensionData {
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

        for conn in self.server.clone().unwrap().connections().await {
            let block_state = block_state.clone();
            let pos = position.clone();
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
            server: Some(server),
            receiver: chan.1,
            sender: chan.0,
            dim_type,
            chunk_generator: |_, _, _| {},
        }
    }

    pub(crate) async fn try_initialize_chunk(&mut self, pos: &Vec2<i32>) {
        if !self.chunks.contains_key(&pos) {
            println!("Initializing: {:?}", pos);

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
            self.chunks.insert(pos.clone(), chunk);

            server.spawn_event(ChunkLoadEvent {
                dimension: Dimension {
                    sender: self.sender.clone(),
                },
                pos: pos.clone(),
            });
        }
    }
}
