use std::marker::PhantomData;

#[allow(dead_code)]
pub struct BlockProperty<T> {
    name: &'static str,
    _phantom: PhantomData<T>,
}

#[allow(dead_code)]
impl<T> BlockProperty<T> {
    pub const fn new(name: &'static str) -> BlockProperty<T> {
        BlockProperty {
            name,
            _phantom: PhantomData,
        }
    }

    const AGE: BlockProperty<i32> = BlockProperty::new("age");
}

#[allow(dead_code)]
pub trait BlockPropertyType
where
    Self: Sized,
{
    fn from_string(str: &str) -> Option<Self>;
    fn into_string(&self) -> String;
}
