use std::collections::HashMap;

use chunk::ChunkSection;

use crate::values::{key::Key, position::Position};

pub mod blocks;
pub mod chunk;

pub struct Dimension {
    name: Key<Dimension>,
    chunks: HashMap<Position<i32>, ChunkSection>,
}
