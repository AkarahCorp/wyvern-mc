#![allow(unused)]

use std::{pin::Pin, sync::OnceLock};

static GLOBAL_RUNTIME: Runtime = Runtime {
    spawn_handler: OnceLock::new(),
};

pub struct Runtime {
    spawn_handler: OnceLock<fn(Pin<Box<dyn Future<Output = ()> + Send>>)>,
}

impl Runtime {
    pub fn spawn<F>(future: F)
    where
        F: Future<Output = ()> + Send + 'static,
    {
        (GLOBAL_RUNTIME
            .spawn_handler
            .get_or_init(|| panic!("No runtime spawn handler set")))(Box::pin(future));
    }

    pub fn set_spawn_handler(function: fn(Pin<Box<dyn Future<Output = ()> + Send>>)) {
        GLOBAL_RUNTIME.spawn_handler.set(function);
    }

    #[cfg(feature = "rt-tokio")]
    pub fn tokio() {
        GLOBAL_RUNTIME.spawn_handler.set(|future| {
            tokio::spawn(future);
        });
    }
}
