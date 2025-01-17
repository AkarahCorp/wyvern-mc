use std::collections::HashMap;

use chunk::ChunkSection;

use crate::values::{key::Key, position::Position};

pub mod chunk;
pub mod blocks;

pub struct Dimension {
    name: Key<Dimension>,
    chunks: HashMap<Position<i32>, ChunkSection>
}