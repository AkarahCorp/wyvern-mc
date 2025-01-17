use std::{fmt::Debug, hash::Hash, marker::PhantomData};

use voxidian_protocol::value::Identifier;

pub struct Key<T> {
    namespace: String,
    path: String,
    _phantom: PhantomData<T>,
}
impl<T> Key<T> {
    pub fn new(namespace: impl Into<String>, path: impl Into<String>) -> Key<T> {
        Key {
            namespace: namespace.into(),
            path: path.into(),
            _phantom: PhantomData::default(),
        }
    }

    pub fn namespace(&self) -> &str {
        &self.namespace
    }

    pub fn path(&self) -> &str {
        &self.path
    }
}

impl<T> From<Identifier> for Key<T> {
    fn from(value: Identifier) -> Self {
        Key::new(value.namespace, value.path)
    }
}

impl<T> From<Key<T>> for Identifier {
    fn from(value: Key<T>) -> Self {
        Identifier::new(value.namespace, value.path)
    }
}

impl<T> Eq for Key<T> {}

impl<T> PartialEq for Key<T> {
    fn eq(&self, other: &Self) -> bool {
        self.namespace == other.namespace && self.path == other.path
    }
}

impl<T> Hash for Key<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.namespace().hash(state);
        self.path().hash(state);
    }
}

impl<T> Debug for Key<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.namespace(), self.path())
    }
}

impl<T> Clone for Key<T> {
    fn clone(&self) -> Self {
        Self {
            namespace: self.namespace.clone(),
            path: self.path.clone(),
            _phantom: self._phantom.clone(),
        }
    }
}
