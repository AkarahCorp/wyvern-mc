use std::{any::{Any, TypeId}, collections::HashMap};

pub struct TypeMap {
    inner: HashMap<TypeId, Box<dyn Any>>
}

impl TypeMap {
    pub fn new() -> Self {
        TypeMap { inner: HashMap::new() }
    }

    pub fn get<T: 'static>(&self) -> Option<&T> {
        self.inner.get(&TypeId::of::<T>()).map(|x| x.downcast_ref::<T>().unwrap())
    }

    pub fn insert<T: 'static>(&mut self, value: T) {
        self.inner.insert(TypeId::of::<T>(), Box::new(value));
    }
}