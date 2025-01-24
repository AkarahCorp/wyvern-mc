use tokio::sync::{mpsc::Sender, oneshot};
use voxidian_protocol::value::DimType;

use crate::{
    server::Server,
    values::{Key, Position},
};

use super::{blocks::BlockState, chunk::ChunkSection, message::DimensionMessage};

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Dimension {
    pub(crate) server: Server,
    pub(crate) tx: Sender<DimensionMessage>,
}

impl Dimension {
    pub async fn get_chunk_at(&self, pos: Position<i32>) -> ChunkSection {
        let (tx, rx) = oneshot::channel();
        self.tx
            .send(DimensionMessage::GetChunkSection(pos, tx))
            .await
            .unwrap();
        rx.await.unwrap()
    }

    pub async fn set_block_at(&self, pos: Position<i32>, block: BlockState) {
        self.tx
            .send(DimensionMessage::SetBlockAt(pos, block))
            .await
            .unwrap();
    }

    pub async fn get_block_at(&self, pos: Position<i32>) -> BlockState {
        let (tx, rx) = oneshot::channel();
        self.tx
            .send(DimensionMessage::GetBlockAt(pos, tx))
            .await
            .unwrap();
        rx.await.unwrap()
    }

    pub async fn get_dimension_type(&self) -> Key<DimType> {
        let (tx, rx) = oneshot::channel();
        self.tx
            .send(DimensionMessage::GetDimensionType(tx))
            .await
            .unwrap();
        rx.await.unwrap()
    }
}
