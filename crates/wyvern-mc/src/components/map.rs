use std::{any::Any, collections::HashMap};

use crate::actors::{ActorError, ActorResult};

use super::DataComponentType;

pub struct DataComponentMap {
    inner: HashMap<u32, Box<dyn Any>>,
}

impl DataComponentMap {
    pub fn new() -> DataComponentMap {
        DataComponentMap {
            inner: HashMap::new(),
        }
    }

    pub fn set<T: 'static>(&mut self, kind: DataComponentType<T>, value: T) {
        self.inner.insert(kind.id(), Box::new(value));
    }

    pub fn with<T: 'static>(mut self, kind: DataComponentType<T>, value: T) -> Self {
        self.inner.insert(kind.id(), Box::new(value));
        self
    }

    pub fn get<T: 'static + Clone>(&self, kind: DataComponentType<T>) -> ActorResult<T> {
        self.inner
            .get(&kind.id())
            .map(|x| x.downcast_ref::<T>().unwrap())
            .cloned()
            .ok_or(ActorError::ComponentNotFound)
    }
}

impl Default for DataComponentMap {
    fn default() -> Self {
        Self::new()
    }
}
