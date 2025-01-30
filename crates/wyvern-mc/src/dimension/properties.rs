use std::marker::PhantomData;

pub struct BlockProperty<T> {
    name: &'static str,
    _phantom: PhantomData<T>,
}

impl<T> BlockProperty<T> {
    pub const fn new(name: &'static str) -> BlockProperty<T> {
        BlockProperty {
            name,
            _phantom: PhantomData,
        }
    }

    const AGE: BlockProperty<i32> = BlockProperty::new("age");
}

pub trait BlockPropertyType
where
    Self: Sized,
{
    fn from_string(str: &str) -> Option<Self>;
    fn into_string(&self) -> String;
}
