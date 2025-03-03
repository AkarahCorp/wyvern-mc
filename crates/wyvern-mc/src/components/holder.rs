use crate::actors::ActorResult;

use super::{ComponentElement, DataComponentMap, DataComponentType};

pub trait DataComponentHolder {
    fn component_map(&self) -> &DataComponentMap;
    fn component_map_mut(&mut self) -> &mut DataComponentMap;

    fn set<T: 'static + ComponentElement>(&mut self, kind: DataComponentType<T>, value: T) {
        self.component_map_mut().set(kind, value);
    }

    fn with<T: 'static + ComponentElement>(mut self, kind: DataComponentType<T>, value: T) -> Self
    where
        Self: Sized,
    {
        self.component_map_mut().set(kind, value);
        self
    }

    fn get<T: 'static + ComponentElement>(&self, kind: DataComponentType<T>) -> ActorResult<T> {
        self.component_map().get(kind)
    }
}
