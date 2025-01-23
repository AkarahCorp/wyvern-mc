use tokio::sync::{mpsc::Sender, oneshot};
use voxidian_protocol::value::DimType;

use crate::{
    server::server::Server,
    values::{key::Key, position::Position},
};

use super::{chunk::ChunkSection, message::DimensionMessage};

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Dimension {
    pub(crate) server: Server,
    pub(crate) tx: Sender<DimensionMessage>,
}

impl Dimension {
    pub async fn get_chunk_at(&self, pos: Position<i32>) -> Option<ChunkSection> {
        let (tx, rx) = oneshot::channel();
        self.tx
            .send(DimensionMessage::GetChunkSection(pos, tx))
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
