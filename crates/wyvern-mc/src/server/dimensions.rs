use std::collections::HashMap;

use crate::{dimension::Dimension, values::Key};

pub(crate) struct DimensionContainer {
    pub(crate) dimensions: HashMap<Key<Dimension>, Dimension>,
}

#[allow(unused)]
impl DimensionContainer {
    pub(crate) fn get(&self, key: &Key<Dimension>) -> Option<&Dimension> {
        self.dimensions.get(key)
    }

    pub(crate) fn insert(&mut self, key: Key<Dimension>, dim: Dimension) {
        self.dimensions.insert(key, dim);
    }

    pub(crate) fn dimensions(&self) -> impl Iterator<Item = &Dimension> {
        self.dimensions.values()
    }

    pub(crate) fn dimensions_mut(&mut self) -> impl Iterator<Item = &mut Dimension> {
        self.dimensions.values_mut()
    }

    pub(crate) fn assert_root_dim_exists(&self) {
        if !self.dimensions.contains_key(&Key::new("wyvern", "root")) {
            println!(
                "\n\nServer Setup Error\n\nPlease define a root dimension with ServerBuilder::root_dimension\nThis is the dimension where players will spawn.\n\n"
            );
            std::process::exit(0)
        }
    }
}
