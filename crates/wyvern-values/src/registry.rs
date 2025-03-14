use voxidian_protocol::registry::{RegEntry, Registry as PtcRegistry};

use super::Id;

pub struct Registry<T> {
    pub(crate) inner: PtcRegistry<T>,
}

impl<T: Clone> Clone for Registry<T> {
    fn clone(&self) -> Self {
        let mut reg = Registry::new();
        for (key, value) in self.entries() {
            reg.insert(key.clone(), value.clone());
        }
        reg
    }
}

impl<T> Registry<T> {
    pub fn new() -> Registry<T> {
        Registry {
            inner: PtcRegistry::new(),
        }
    }
    pub fn insert(&mut self, key: Id, value: T) {
        self.inner.insert(key.into(), value);
    }

    pub fn get(&self, key: Id) -> Option<&T> {
        self.inner.get(&key.into())
    }

    pub fn keys(&self) -> impl Iterator<Item = Id> {
        self.inner.keys().map(|x| x.clone().into())
    }

    pub fn entries(&self) -> impl Iterator<Item = (Id, &T)> {
        self.inner.entries().map(|x| (x.0.clone().into(), x.1))
    }

    pub fn clear(&mut self) {
        self.inner.clear();
    }

    pub fn get_entry(&self, key: Id) -> Option<RegEntry<T>> {
        self.inner.get_entry(&key.into())
    }

    pub fn inner(&self) -> &PtcRegistry<T> {
        &self.inner
    }

    pub fn inner_mut(&mut self) -> &mut PtcRegistry<T> {
        &mut self.inner
    }
}

impl<T> Default for Registry<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> From<PtcRegistry<T>> for Registry<T> {
    fn from(value: PtcRegistry<T>) -> Registry<T> {
        Registry { inner: value }
    }
}
