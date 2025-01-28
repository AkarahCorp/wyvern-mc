use std::collections::HashMap;

use crate::{
    dimension::{Dimension, DimensionData},
    values::Key,
};

pub struct DimensionContainer {
    pub(crate) dimensions: HashMap<Key<Dimension>, DimensionData>,
}

impl DimensionContainer {
    pub fn get(&self, key: &Key<Dimension>) -> Option<&DimensionData> {
        self.dimensions.get(key)
    }

    pub fn insert(&mut self, key: Key<Dimension>, dim: DimensionData) {
        self.dimensions.insert(key, dim);
    }

    pub fn dimensions(&self) -> impl Iterator<Item = &DimensionData> {
        self.dimensions.values()
    }

    pub fn dimensions_mut(&mut self) -> impl Iterator<Item = &mut DimensionData> {
        self.dimensions.values_mut()
    }

    pub fn assert_root_dim_exists(&self) {
        if !self.dimensions.contains_key(&Key::new("wyvern", "root")) {
            println!(
                "\n\nServer Setup Error\n\nPlease define a root dimension with ServerBuilder::root_dimension\nThis is the dimension where players will spawn.\n\n"
            );
            std::process::exit(0)
        }
    }
}
