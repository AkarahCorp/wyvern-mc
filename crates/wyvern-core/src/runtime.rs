#![allow(unused)]

use std::{
    collections::VecDeque,
    pin::Pin,
    sync::{LazyLock, Mutex, OnceLock},
    task::{Context, Poll},
    thread::Builder,
};

use async_executor::{Executor, Task};
use flume::{Receiver, Sender};
use lockfree::queue::Queue;

use crate::actors::ActorResult;

pub(crate) static GLOBAL_RUNTIME: Runtime = Runtime {
    executor: Executor::new(),
};

pub struct Runtime {
    pub(crate) executor: Executor<'static>,
}

impl Runtime {
    pub fn spawn_actor<F>(func: F, name: impl Into<String>)
    where
        F: FnOnce() + Send + 'static,
    {
        let builder = Builder::new().name(name.into()).spawn(func).unwrap();
    }

    pub fn executor(&self) -> &'static Executor<'static> {
        &GLOBAL_RUNTIME.executor
    }

    pub fn spawn_task<F>(fut: F)
    where
        F: Future<Output = ActorResult<()>> + Send + Sync + 'static,
    {
        GLOBAL_RUNTIME.executor.spawn(fut).detach();
    }

    pub fn run_async<T: Send + 'static, F>(fut: F) -> Task<ActorResult<T>>
    where
        F: Future<Output = ActorResult<T>> + Send + Sync + 'static,
    {
        GLOBAL_RUNTIME.executor.spawn(fut)
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

pub struct NeverYield;

impl Future for NeverYield {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        cx.waker().wake_by_ref();
        Poll::Pending
    }
}
