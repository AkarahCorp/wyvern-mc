use std::{
    marker::PhantomData,
    sync::atomic::{AtomicU32, Ordering},
};

use crate::values::Id;

static COMPONENT_TYPE_INDEX: AtomicU32 = AtomicU32::new(0);

pub struct DataComponentType<T> {
    id: u32,
    name: Id,
    _phantom: PhantomData<T>,
}

impl<T> DataComponentType<T> {
    pub fn new(name: Id) -> DataComponentType<T> {
        DataComponentType {
            id: COMPONENT_TYPE_INDEX.fetch_add(1, Ordering::Relaxed),
            name,
            _phantom: PhantomData,
        }
    }

    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn name(&self) -> &Id {
        &self.name
    }
}
