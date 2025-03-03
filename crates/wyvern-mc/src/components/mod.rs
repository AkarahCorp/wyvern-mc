mod types;
use std::{
    any::{Any, type_name_of_val},
    fmt::Debug,
};

use dyn_clone::DynClone;
pub use types::*;
mod map;
pub use map::*;
mod holder;
pub use holder::*;

pub trait ComponentElement: Any + Sync + Send + DynClone + Debug {
    fn element_type_name(&self) -> String;
}

impl<T: Any + Sync + Send + DynClone + Debug> ComponentElement for T {
    fn element_type_name(&self) -> String {
        type_name_of_val(self).to_string()
    }
}
