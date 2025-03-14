use std::collections::HashMap;

use crate::{dimension::Dimension, values::Id};

pub(crate) struct DimensionContainer {
    pub(crate) dimensions: HashMap<Id, Dimension>,
}

#[allow(unused)]
impl DimensionContainer {
    pub(crate) fn get(&self, key: &Id) -> Option<&Dimension> {
        self.dimensions.get(key)
    }

    pub(crate) fn insert(&mut self, key: Id, dim: Dimension) {
        self.dimensions.insert(key, dim);
    }

    pub(crate) fn dimensions(&self) -> impl Iterator<Item = &Dimension> {
        self.dimensions.values()
    }

    pub(crate) fn dimensions_mut(&mut self) -> impl Iterator<Item = &mut Dimension> {
        self.dimensions.values_mut()
    }

    pub(crate) fn assert_root_dim_exists(&self) {
        if !self.dimensions.contains_key(&Id::new("wyvern", "root")) {
            log::error!(
                "Server Setup Error
                
                Please define a root dimension with ServerBuilder::root_dimension
                This is the dimension where players will spawn."
            );
            std::process::exit(0)
        }
    }
}
