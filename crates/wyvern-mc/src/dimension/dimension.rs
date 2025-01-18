use std::collections::HashMap;

use tokio::sync::mpsc::{Sender, channel};

use crate::values::key::Key;

use super::{DimensionData, message::DimensionMessage};

#[allow(dead_code)]
pub struct Dimension {
    pub(crate) tx: Sender<DimensionMessage>,
}

impl Dimension {
    pub fn root() -> DimensionData {
        let chan = channel(1024);
        DimensionData {
            name: Key::new("wyvern", "root"),
            chunks: HashMap::new(),
            server: None,
            rx: chan.1,
            tx: chan.0,
        }
    }
}
