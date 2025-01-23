use std::{
    any::{Any, TypeId},
    collections::{HashMap, hash_map::Values},
};

#[derive(Debug)]
pub struct TypeMap {
    inner: HashMap<TypeId, Box<dyn Any + Send + Sync>>,
}

impl Default for TypeMap {
    fn default() -> Self {
        Self::new()
    }
}

impl TypeMap {
    pub fn new() -> Self {
        TypeMap {
            inner: HashMap::new(),
        }
    }

    pub fn get<T: Send + Sync + 'static>(&self) -> Option<&T> {
        self.inner
            .get(&TypeId::of::<T>())
            .map(|x| x.downcast_ref::<T>().unwrap())
    }

    pub fn insert<T: Send + Sync + 'static>(&mut self, value: T) {
        self.inner.insert(TypeId::of::<T>(), Box::new(value));
    }

    pub fn values(&self) -> Values<TypeId, Box<dyn Any + Send + Sync>> {
        self.inner.values()
    }
}
