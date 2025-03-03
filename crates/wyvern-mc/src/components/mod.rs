mod types;
use std::{any::Any, fmt::Debug};

use dyn_clone::DynClone;
pub use types::*;
mod map;
pub use map::*;
mod holder;
pub use holder::*;

pub trait ComponentElement: Any + Sync + Send + DynClone + Debug {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

impl<T: Any + Sync + Send + DynClone + Debug> ComponentElement for T {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
