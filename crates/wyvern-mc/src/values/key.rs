use std::marker::PhantomData;

use voxidian_protocol::value::Identifier;

pub struct Key<T> {
    namespace: String,
    path: String,
    _phantom: PhantomData<T>
}

impl<T> Clone for Key<T> {
    fn clone(&self) -> Self {
        Self { namespace: self.namespace.clone(), path: self.path.clone(), _phantom: self._phantom.clone() }
    }
}

impl<T> Key<T> {
    pub fn new(namespace: impl Into<String>, path: impl Into<String>) -> Key<T> {
        Key {
            namespace: namespace.into(),
            path: path.into(),
            _phantom: PhantomData::default()
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