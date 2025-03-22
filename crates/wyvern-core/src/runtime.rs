#![allow(unused)]

use std::{
    pin::Pin,
    sync::{LazyLock, OnceLock},
    task::{Context, Poll},
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

    pub async fn yield_now() {
        YieldNow(false).await
    }
}

// thanks async-std
pub struct YieldNow(bool);

impl Future for YieldNow {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if !self.0 {
            self.0 = true;

            cx.waker().wake_by_ref();

            Poll::Pending
        } else {
            Poll::Ready(())
        }
    }
}
