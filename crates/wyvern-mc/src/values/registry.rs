use voxidian_protocol::registry::{RegEntry, Registry as PtcRegistry};

use super::Key;

pub struct Registry<T> {
    pub(crate) inner: PtcRegistry<T>,
}

impl<T> Registry<T> {
    pub fn insert(&mut self, key: Key<T>, value: T) {
        self.inner.insert(key.into(), value);
    }

    pub fn get(&self, key: Key<T>) -> Option<&T> {
        self.inner.get(&key.into())
    }

    pub fn keys(&self) -> impl Iterator<Item = Key<T>> {
        self.inner.keys().map(|x| x.clone().into())
    }

    pub fn entries(&self) -> impl Iterator<Item = (Key<T>, &T)> {
        self.inner.entries().map(|x| (x.0.clone().into(), x.1))
    }

    pub fn clear(&mut self) {
        self.inner.clear();
    }

    pub(crate) fn make_entry(&self, key: Key<T>) -> Option<RegEntry<T>> {
        self.inner.make_entry(&key.into())
    }
}

impl<T> From<PtcRegistry<T>> for Registry<T> {
    fn from(value: PtcRegistry<T>) -> Registry<T> {
        Registry { inner: value }
    }
}
