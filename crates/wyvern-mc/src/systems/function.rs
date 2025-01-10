use std::marker::PhantomData;

pub struct FunctionSystem<Input, F> {
    pub(crate) f: F,
    pub(crate) _phantom: PhantomData<fn() -> Input>,
}