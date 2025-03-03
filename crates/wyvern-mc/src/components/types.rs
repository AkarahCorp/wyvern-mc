use std::marker::PhantomData;

use crate::values::Id;

#[derive(PartialEq, Hash, Clone, Debug)]
pub struct DataComponentType<T> {
    id: u64,
    name: Id,
    _phantom: PhantomData<T>,
}

impl<T> DataComponentType<T> {
    pub const fn new(id: u64, name: Id) -> DataComponentType<T> {
        DataComponentType {
            id,
            name,
            _phantom: PhantomData,
        }
    }

    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn name(&self) -> &Id {
        &self.name
    }
}
