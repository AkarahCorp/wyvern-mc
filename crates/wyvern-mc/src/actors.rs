use crate::runtime::Runtime;

pub trait Actor {
    fn handle_messages(&mut self) -> impl Future<Output = ()> + Send + Sync;

    fn intertwine<F: AsyncFnOnce() + Send + Sync>(&mut self, f: F) -> impl Future<Output = ()> {
        async move {
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

pub type ActorResult<T> = Result<T, ActorError>;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum ActorError {
    ActorDoesNotExist,
    ActorIsNotLoaded,
    IndexOutOfBounds,
    BadRequest,
}
