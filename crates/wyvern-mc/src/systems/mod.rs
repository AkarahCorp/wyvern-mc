use std::pin::Pin;

pub mod events;
pub mod function;
pub mod intos;
pub mod parameters;
pub mod system;
pub mod typemap;

pub(crate) type BoxedFuture = Pin<Box<dyn Future<Output = ()> + Send + Sync + 'static>>;
