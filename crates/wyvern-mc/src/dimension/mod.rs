use std::collections::HashMap;

use chunk::ChunkSection;
use message::DimensionMessage;
use tokio::sync::mpsc::{Receiver, Sender, channel};

use crate::{
    server::server::Server,
    values::{key::Key, position::Position},
};

pub mod blocks;
pub mod chunk;
pub(crate) mod dimension;
pub mod message;

pub use dimension::*;

#[allow(dead_code)]
pub struct DimensionData {
    pub(crate) name: Key<DimensionData>,
    pub(crate) chunks: HashMap<Position<i32>, ChunkSection>,
    pub(crate) server: Option<Server>,
    pub(crate) rx: Receiver<DimensionMessage>,
    pub(crate) tx: Sender<DimensionMessage>,
}

impl DimensionData {
    pub(crate) fn new(name: Key<DimensionData>, server: Server) -> DimensionData {
        let chan = channel(1024);
        DimensionData {
            name,
            chunks: HashMap::new(),
            server: Some(server),
            rx: chan.1,
            tx: chan.0,
        }
    }

    pub async fn handle_messages(mut self) {
        loop {
            match self.rx.recv().await {
                Some(msg) => match msg {},
                None => {}
            }
        }
    }
}
