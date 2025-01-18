use std::collections::HashMap;


use crate::{
    dimension::Dimension,
    values::key::Key,
};

pub struct DimensionContainer {
    pub(crate) dimensions: HashMap<Key<Dimension>, Dimension>,
}

impl DimensionContainer {
    pub fn get(&self, key: &Key<Dimension>) -> Option<&Dimension> {
        self.dimensions.get(&key)
    }

    pub fn insert(&mut self, key: Key<Dimension>, dim: Dimension) {
        self.dimensions.insert(key, dim);
    }

    pub fn dimensions(&self) -> impl Iterator<Item = &Dimension> {
        self.dimensions.values()
    }

    pub fn dimensions_mut(&mut self) -> impl Iterator<Item = &mut Dimension> {
        self.dimensions.values_mut()
    }

    pub fn assert_root_dim_exists(&self) {
        if !self.dimensions.contains_key(&Key::new("wyvern", "root")) {
            println!(
                "\n\n{}\n\n{}\n{}\n\n",
                "Server Setup Error",
                "Please define a root dimension with ServerBuilder::root_dimension",
                "This is the dimension where players will spawn."
            );
            std::process::exit(0)
        }
    }
}
