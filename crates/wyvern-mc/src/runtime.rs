#![allow(unused)]

use std::{pin::Pin, sync::OnceLock};

static GLOBAL_RUNTIME: Runtime = Runtime {
    spawn_handler: OnceLock::new(),
};

pub struct Runtime {
    spawn_handler: OnceLock<fn(Pin<Box<dyn Future<Output = ()> + Send>>)>,
}

impl Runtime {
    pub fn spawn<F>(func: F)
    where
        F: FnOnce() + Send + 'static,
    {
        std::thread::spawn(func);
    }
}
