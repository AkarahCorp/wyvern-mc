use std::marker::PhantomData;

use crate::values::Id;

#[derive(PartialEq, Hash, Clone, Debug)]
pub struct DataComponentType<T> {
    name: Id,
    _phantom: PhantomData<T>,
}

impl<T> DataComponentType<T> {
    pub const fn new(name: Id) -> DataComponentType<T> {
        DataComponentType {
            name,
            _phantom: PhantomData,
        }
    }

    pub fn name(&self) -> &Id {
        &self.name
    }

    pub fn into_name(self) -> Id {
        self.name
    }
}
