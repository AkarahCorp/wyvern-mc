use voxidian_protocol::autogenerated::block_states::{BLOCK_STATE_TO_ID, ID_TO_BLOCK_STATE};
use voxidian_protocol::value::BlockState as ProtocolState;

use crate::values::Key;

pub struct Block {}

#[derive(Debug, Clone, PartialEq)]
pub struct BlockState {
    block: Key<Block>,
    state: Vec<(String, String)>,
}

impl BlockState {
    pub fn new(id: Key<Block>) -> BlockState {
        BlockState {
            block: id,
            state: Vec::new(),
        }
    }

    pub fn with_property(mut self, key: &str, value: &str) -> Self {
        if let Some(index) = self.state.iter().map(|x| &x.0).position(|x| x == &key) {
            self.state.remove(index);
        }
        self.state.push((key.into(), value.into()));
        self
    }

    pub fn insert_property(&mut self, key: &str, value: &str) {
        if let Some(index) = self.state.iter().map(|x| &x.0).position(|x| x == &key) {
            self.state.remove(index);
        }
        self.state.push((key.into(), value.into()));
    }

    pub fn protocol_id(&self) -> i32 {
        *BLOCK_STATE_TO_ID.get(&self.into()).unwrap()
    }

    pub fn from_protocol_id(id: i32) -> Self {
        Self::from(ID_TO_BLOCK_STATE.get(&id).unwrap())
    }
}

impl From<&ProtocolState> for BlockState {
    fn from(value: &ProtocolState) -> Self {
        BlockState {
            block: value.id.clone().into(),
            state: value.properties.clone(),
        }
    }
}

impl From<&BlockState> for ProtocolState {
    fn from(value: &BlockState) -> Self {
        ProtocolState {
            id: value.block.clone().into(),
            properties: value.state.clone(),
        }
    }
}
