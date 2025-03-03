#![allow(unused)]

use std::{
    pin::Pin,
    sync::{LazyLock, OnceLock},
};

use flume::{Receiver, Sender};

use crate::actors::ActorResult;

static GLOBAL_RUNTIME: Runtime = Runtime {
    tasks: OnceLock::new(),
};

pub struct Runtime {
    tasks: OnceLock<Sender<Box<dyn FnOnce() -> ActorResult<()> + Send>>>,
}

impl Runtime {
    pub fn spawn_actor<F>(func: F)
    where
        F: FnOnce() + Send + 'static,
    {
        std::thread::spawn(func);
    }

    pub fn spawn_task<F>(func: F)
    where
        F: FnOnce() -> ActorResult<()> + Send + 'static,
    {
        let sender = GLOBAL_RUNTIME.tasks.get_or_init(|| {
            let chan = flume::unbounded();
            for _ in 0..std::thread::available_parallelism()
                .expect("Multithreaded system is required")
                .into()
            {
                let recv: Receiver<Box<dyn FnOnce() -> ActorResult<()> + Send>> = chan.1.clone();
                std::thread::spawn(move || {
                    loop {
                        match recv.recv() {
                            Ok(task) => {
                                task();
                            }
                            Err(_) => break,
                        }
                    }
                });
            }
            chan.0
        });
        sender.send(Box::new(func));
    }
}
