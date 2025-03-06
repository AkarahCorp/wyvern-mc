use std::sync::Arc;

use dyn_clone::clone_box;
use rustc_hash::FxHashMap;

use crate::{
    actors::{ActorError, ActorResult},
    values::Id,
};

use super::{ComponentElement, DataComponentType};

#[derive(Clone, Debug)]
pub struct DataComponentMap {
    pub(crate) inner: FxHashMap<Id, Arc<dyn ComponentElement>>,
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

    pub fn keys(&self) -> Vec<&Id> {
        self.inner.keys().collect()
    }
}

impl Default for DataComponentMap {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use std::time::Instant;

    use rustc_hash::FxHashMap;

    use crate::item::ItemComponents;

    use super::DataComponentMap;

    #[test]
    fn benchmark_maps() {
        let t1 = Instant::now();
        let mut hm = FxHashMap::default();
        hm.insert("minecraft:damage", 10);
        hm.insert("minecraft:max_damage", 20);

        hm.get("minecraft:damage").unwrap();
        hm.get("minecraft:max_damage").unwrap();

        let t2 = Instant::now();
        let mut cm = DataComponentMap::new();
        cm.set(ItemComponents::DAMAGE, 10);
        cm.set(ItemComponents::MAX_DAMAGE, 20);

        cm.get(ItemComponents::DAMAGE).unwrap();
        cm.get(ItemComponents::MAX_DAMAGE).unwrap();
        let t3 = Instant::now();

        eprintln!("HashMap: {:?}\nDataComponentMap:{:?}", t2 - t1, t3 - t2)
    }
}
