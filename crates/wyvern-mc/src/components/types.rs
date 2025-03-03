use std::{
    marker::PhantomData,
    sync::atomic::{AtomicU32, Ordering},
};

static COMPONENT_TYPE_INDEX: AtomicU32 = AtomicU32::new(0);

pub struct DataComponentType<T> {
    #[allow(unused)]
    id: u32,
    _phantom: PhantomData<T>,
}

impl<T> DataComponentType<T> {
    pub fn new() -> DataComponentType<T> {
        DataComponentType {
            id: COMPONENT_TYPE_INDEX.fetch_add(1, Ordering::Relaxed),
            _phantom: PhantomData,
        }
    }

    pub fn id(&self) -> u32 {
        self.id
    }
}

impl<T> Default for DataComponentType<T> {
    fn default() -> Self {
        Self::new()
    }
}
