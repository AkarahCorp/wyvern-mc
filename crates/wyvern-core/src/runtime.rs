#![allow(unused)]

use std::{
    pin::Pin,
    sync::{LazyLock, OnceLock},
    thread::Builder,
};

use async_executor::Executor;
use flume::{Receiver, Sender};

use crate::actors::ActorResult;

static GLOBAL_RUNTIME: Runtime = Runtime {
    tasks: OnceLock::new(),
};

pub(crate) static GLOBAL_EXECUTOR: Executor = Executor::new();

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

    pub fn spawn_task<F>(fut: F)
    where
        F: Future<Output = ActorResult<()>> + Send + 'static,
    {
        GLOBAL_EXECUTOR.spawn(fut).detach();
    }
}
