use std::pin::Pin;

pub mod scheduler;
pub mod system;
pub mod typemap;
pub mod function;
pub mod intos;
pub mod parameters;
pub mod events;

pub(crate) type BoxedFuture = Pin<Box<dyn Future<Output = ()> + Send + Sync + 'static>>;