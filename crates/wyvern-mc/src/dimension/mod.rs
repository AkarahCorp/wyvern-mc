use std::collections::HashMap;

use blocks::BlockState;
use chunk::ChunkSection;
use tokio::sync::mpsc::{Sender, channel};
use voxidian_protocol::{
    packet::{Stage, s2c::play::BlockUpdateS2CPlayPacket},
    registry::RegEntry,
    value::{BlockPos, DimType},
};

use crate::{
    server::Server,
    systems::{
        events::ChunkLoadEvent,
        parameters::{Event, Param},
        typemap::TypeMap,
    },
    values::{Key, Position},
};

pub mod blocks;
pub mod chunk;

#[allow(dead_code)]
#[crate::actor(Dimension, DimensionMessage)]
pub struct DimensionData {
    #[allow(unused)]
    pub(crate) name: Key<DimensionData>,
    pub(crate) chunks: HashMap<Position<i32>, ChunkSection>,
    pub(crate) server: Option<Server>,
    pub(crate) sender: Sender<DimensionMessage>,
    pub(crate) dim_type: Key<DimType>,
}

#[crate::message(Dimension, DimensionMessage)]
impl DimensionData {
    #[GetServer]
    pub async fn get_server(&self) -> Option<Server> {
        self.server.clone()
    }

    #[GetChunkSection]
    pub async fn get_chunk_section(&mut self, position: Position<i32>) -> ChunkSection {
        self.try_initialize_chunk(&position).await;

        let chunk = self.chunks.get(&position).unwrap();
        chunk.clone()
    }

    #[SetBlock]
    pub async fn set_block(&mut self, position: Position<i32>, block_state: BlockState) {
        let chunk_pos = position.map_into_coords(|x| x / 16);
        let pos_in_chunk = position.map_into_coords(|x| {
            let x = x.abs() % 16;
            x as usize
        });

        self.try_initialize_chunk(&chunk_pos).await;

        let chunk = self.chunks.get_mut(&chunk_pos).unwrap();
        chunk.set_block_at(pos_in_chunk, block_state.clone());

        let pos = chunk_pos.clone().map(|x| x * 16) + pos_in_chunk.clone().map(|x| *x as i32);

        for conn in self.server.clone().unwrap().connections().await {
            let block_state = block_state.clone();
            let pos = pos.clone();
            let conn = conn.clone();
            // TODO: DEADLOCKS HERE (do CTRL+SHIFT+F for other point)
            tokio::spawn(async move {
                if conn.is_loaded_in_world().await {
                    conn.write_packet(BlockUpdateS2CPlayPacket {
                        pos: BlockPos::new(*pos.x(), *pos.y(), *pos.z()),
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
    pub async fn get_block_at(&mut self, position: Position<i32>) -> BlockState {
        let chunk = position.map_into_coords(|x| x / 16);
        let pos_in_chunk = position.map_into_coords(|x| (x % 16) as usize);

        self.try_initialize_chunk(&chunk).await;

        let chunk = self.chunks.get_mut(&chunk).unwrap();
        chunk.get_block_at(pos_in_chunk)
    }

    #[GetDimType]
    pub async fn get_dimension_type(&mut self) -> Key<DimType> {
        self.dim_type.clone()
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
        }
    }

    pub(crate) async fn try_initialize_chunk(&mut self, pos: &Position<i32>) {
        if !self.chunks.contains_key(&pos) {
            println!("Initializing: {:?}", pos);
            self.chunks.insert(pos.clone(), ChunkSection::empty());

            self.server
                .clone()
                .unwrap()
                .fire_systems({
                    let mut map = TypeMap::new();
                    map.insert(Event::<ChunkLoadEvent>::new());
                    map.insert(Param::new(pos.clone()));
                    map.insert(Param::new(Dimension {
                        sender: self.sender.clone(),
                    }));
                    map
                })
                .await;
        }
    }
}
