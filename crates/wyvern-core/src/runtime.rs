#![allow(unused)]

use std::{
    pin::Pin,
    sync::{LazyLock, OnceLock},
    thread::Builder,
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
    pub fn spawn_actor<F>(func: F, name: impl Into<String>)
    where
        F: FnOnce() + Send + 'static,
    {
        let builder = Builder::new().name(name.into()).spawn(func).unwrap();
    }

    pub fn spawn_task<F>(func: F)
    where
        F: FnOnce() -> ActorResult<()> + Send + 'static,
    {
        let sender = GLOBAL_RUNTIME.tasks.get_or_init(|| {
            let chan = flume::unbounded();
            for _ in 0..(std::thread::available_parallelism()
                .expect("Multithreaded system is required")
                .get()
                / 2)
            {
                let recv: Receiver<Box<dyn FnOnce() -> ActorResult<()> + Send>> = chan.1.clone();
                Builder::new()
                    .name("TaskThread".to_string())
                    .spawn(move || {
                        while let Ok(task) = recv.recv() {
                            println!("{:#?}", recv.len());
                            task();
                        }
                    });
            }
            chan.0
        });
        sender.send(Box::new(func));
    }
}
