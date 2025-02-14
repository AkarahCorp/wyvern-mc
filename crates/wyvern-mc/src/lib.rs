#![feature(try_blocks)]
#![allow(clippy::type_complexity)]

pub mod components;
pub mod dimension;
pub mod events;
pub mod future;
pub mod inventory;
pub mod player;
pub mod runtime;
pub mod server;
pub mod values;

pub mod actors {
    use crate::runtime::Runtime;

    pub trait Actor {
        fn handle_messages(&mut self) -> impl Future<Output = ()> + Send + Sync;

        #[allow(async_fn_in_trait)]
        async fn intertwine<F: AsyncFnOnce()>(&mut self, f: F) {
            futures_lite::future::race(
                async move {
                    loop {
                        self.handle_messages().await;
                        Runtime::yield_now().await;
                    }
                },
                async move { f().await },
            )
            .await
        }
    }
}

pub use wyvern_macros::*;
