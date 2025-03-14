use std::sync::Arc;

use dyn_clone::clone_box;
use rustc_hash::FxHashMap;

use wyvern_actors::{ActorError, ActorResult};
use wyvern_values::Id;

use super::{ComponentElement, DataComponentType};

#[derive(Clone, Debug)]
pub struct DataComponentMap {
    pub(crate) inner: FxHashMap<Id, Arc<dyn ComponentElement>>,
}

impl PartialEq for DataComponentMap {
    fn eq(&self, other: &Self) -> bool {
        if self.inner.keys().collect::<Vec<_>>() != other.inner.keys().collect::<Vec<_>>() {
            return false;
        }
        for key in self.inner.keys() {
            let other_value = other.inner.get(key).unwrap();
            if !self.inner.get(key).unwrap().compare(other_value.as_ref()) {
                return false;
            }
        }
        true
    }
}

impl DataComponentMap {
    pub fn new() -> DataComponentMap {
        DataComponentMap {
            inner: FxHashMap::default(),
        }
    }

    pub fn set<T: 'static + ComponentElement>(&mut self, kind: DataComponentType<T>, value: T) {
        self.inner.insert(kind.into_name(), Arc::new(value));
    }

    pub fn with<T: 'static + ComponentElement>(
        mut self,
        kind: DataComponentType<T>,
        value: T,
    ) -> Self {
        self.inner.insert(kind.into_name(), Arc::new(value));
        self
    }

    pub fn get<T: 'static + ComponentElement>(&self, kind: DataComponentType<T>) -> ActorResult<T> {
        self.inner
            .get(&kind.into_name())
            .and_then(|x| (**x).as_any().downcast_ref::<T>())
            .map(|x| clone_box(x))
            .map(|x| *x)
            .ok_or(ActorError::ComponentNotFound)
    }

    pub fn contains(&self, key: &Id) -> bool {
        self.inner.contains_key(key)
    }

    pub fn contains_type<T>(&self, ty: &DataComponentType<T>) -> bool {
        self.inner.contains_key(ty.name())
    }

    pub fn keys(&self) -> Vec<&Id> {
        self.inner.keys().collect()
    }

    pub fn inner(&self) -> &FxHashMap<Id, Arc<dyn ComponentElement>> {
        &self.inner
    }

    pub fn inner_mut(&mut self) -> &mut FxHashMap<Id, Arc<dyn ComponentElement>> {
        &mut self.inner
    }
}

impl Default for DataComponentMap {
    fn default() -> Self {
        Self::new()
    }
}
