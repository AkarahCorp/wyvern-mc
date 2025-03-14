use std::{
    fmt::Debug,
    sync::{Arc, Mutex},
};

#[derive(Clone)]
pub struct Token<T: Clone> {
    inner: Arc<Mutex<T>>,
}

impl<T: Clone> Debug for Token<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Token").finish()
    }
}

impl<T: Clone> Token<T> {
    pub fn new(value: T) -> Token<T> {
        Token {
            inner: Arc::new(Mutex::new(value)),
        }
    }

    pub fn set(&self, value: T) {
        *self.inner.lock().unwrap() = value;
    }

    pub fn get(&self) -> T {
        self.inner.lock().unwrap().clone()
    }
}
