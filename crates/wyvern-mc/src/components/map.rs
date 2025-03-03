use std::{collections::HashMap, sync::Arc};

use dyn_clone::clone_box;

use crate::actors::{ActorError, ActorResult};

use super::{ComponentElement, DataComponentType};

#[derive(Clone, Debug)]
pub struct DataComponentMap {
    inner: HashMap<u64, Arc<dyn ComponentElement>>,
}

impl DataComponentMap {
    pub fn new() -> DataComponentMap {
        DataComponentMap {
            inner: HashMap::new(),
        }
    }

    pub fn set<T: 'static + ComponentElement>(&mut self, kind: DataComponentType<T>, value: T) {
        self.inner.insert(kind.id(), Arc::new(value));
    }

    pub fn with<T: 'static + ComponentElement>(
        mut self,
        kind: DataComponentType<T>,
        value: T,
    ) -> Self {
        self.inner.insert(kind.id(), Arc::new(value));
        self
    }

    pub fn get<T: 'static + ComponentElement>(&self, kind: DataComponentType<T>) -> ActorResult<T> {
        self.inner
            .get(&kind.id())
            .and_then(|x| (**x).as_any().downcast_ref::<T>())
            .map(|x| clone_box(x))
            .map(|x| *x)
            .ok_or(ActorError::ComponentNotFound)
    }
}

impl Default for DataComponentMap {
    fn default() -> Self {
        Self::new()
    }
}
