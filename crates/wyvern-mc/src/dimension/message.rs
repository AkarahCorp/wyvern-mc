use tokio::sync::oneshot::Sender;
use voxidian_protocol::value::DimType;

use crate::values::{Key, Position};

use super::{blocks::BlockState, chunk::ChunkSection};

pub enum DimensionMessage {
    GetChunkSection(Position<i32>, Sender<Option<ChunkSection>>),
    GetDimensionType(Sender<Key<DimType>>),
    SetBlockAt(Position<i32>, BlockState),
    GetBlockAt(Position<i32>, Sender<BlockState>),
}
