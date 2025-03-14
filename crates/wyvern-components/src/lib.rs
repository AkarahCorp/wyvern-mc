use std::{any::Any, fmt::Debug};

use dyn_clone::DynClone;
mod types;
pub use types::*;
mod map;
pub use map::*;
mod holder;
pub use holder::*;
mod patch;
pub use patch::*;

pub trait ComponentElement: Any + Sync + Send + DynClone + Debug {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn compare(&self, other: &dyn ComponentElement) -> bool;
}

impl<T: Any + Sync + Send + DynClone + Debug + PartialEq> ComponentElement for T {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn compare(&self, other: &dyn ComponentElement) -> bool {
        let Some(other) = (*other).as_any().downcast_ref::<T>() else {
            return false;
        };
        self == other
    }
}
