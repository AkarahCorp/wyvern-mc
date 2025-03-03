use crate::actors::ActorResult;

use super::{DataComponentMap, DataComponentType};

pub trait DataComponentHolder {
    fn component_map(&self) -> &DataComponentMap;
    fn component_map_mut(&mut self) -> &mut DataComponentMap;

    fn set<T: 'static>(&mut self, kind: DataComponentType<T>, value: T) {
        self.component_map_mut().set(kind, value);
    }

    fn with<T: 'static>(mut self, kind: DataComponentType<T>, value: T) -> Self
    where
        Self: Sized,
    {
        self.component_map_mut().set(kind, value);
        self
    }

    fn get<T: 'static + Clone>(&self, kind: DataComponentType<T>) -> ActorResult<T> {
        self.component_map().get(kind)
    }
}
