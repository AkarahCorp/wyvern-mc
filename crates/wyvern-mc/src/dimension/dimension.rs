use tokio::sync::{mpsc::Sender, oneshot};

use crate::{server::server::Server, values::position::Position};

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
}
