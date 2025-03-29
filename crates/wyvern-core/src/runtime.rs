#![allow(unused)]

use std::{
    collections::VecDeque,
    pin::Pin,
    sync::{LazyLock, Mutex, OnceLock},
    task::{Context, Poll},
    thread::Builder,
};

use async_executor::Executor;
use flume::{Receiver, Sender};
use lockfree::queue::Queue;

use crate::actors::ActorResult;

pub(crate) static GLOBAL_RUNTIME: LazyLock<Runtime> = const {
    LazyLock::new(|| {
        let chan = flume::unbounded();
        Runtime {
            sender: chan.0,
            receiver: chan.1,
        }
    })
};

pub struct Runtime {
    pub(crate) sender: Sender<Pin<Box<dyn Future<Output = ActorResult<()>> + Send + Sync>>>,
    pub(crate) receiver: Receiver<Pin<Box<dyn Future<Output = ActorResult<()>> + Send + Sync>>>,
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
        F: Future<Output = ActorResult<()>> + Send + Sync + 'static,
    {
        GLOBAL_RUNTIME.sender.send(Box::pin(fut));
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
