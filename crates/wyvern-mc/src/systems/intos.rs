use std::marker::PhantomData;

use super::{function::FunctionSystem, parameters::SystemParameter, system::System};

pub trait IntoSystem<Input> {
    type System: System;

    fn into_system(self) -> Self::System;
}

impl<Fut, F: FnMut() -> Fut> IntoSystem<()> for F
where
    Fut: Future<Output = ()> + Send + Sync + 'static,
{
    type System = FunctionSystem<(), Self>;

    fn into_system(self) -> Self::System {
        FunctionSystem {
            f: self,
            _phantom: PhantomData::default(),
        }
    }
}

impl<Fut, F: FnMut(T1) -> Fut, T1> IntoSystem<(T1,)> for F
where
    Fut: Future<Output = ()> + Send + Sync + 'static,
    T1: SystemParameter + Clone + 'static,
{
    type System = FunctionSystem<(T1,), Self>;

    fn into_system(self) -> Self::System {
        FunctionSystem {
            f: self,
            _phantom: PhantomData::default(),
        }
    }
}

impl<Fut, F: FnMut(T1, T2) -> Fut, T1, T2> IntoSystem<(T1, T2)> for F
where
    Fut: Future<Output = ()> + Send + Sync + 'static,
    T1: SystemParameter + Clone + 'static,
    T2: SystemParameter + Clone + 'static,
{
    type System = FunctionSystem<(T1, T2), Self>;

    fn into_system(self) -> Self::System {
        FunctionSystem {
            f: self,
            _phantom: PhantomData::default(),
        }
    }
}

impl<Fut, F: FnMut(T1, T2, T3) -> Fut, T1, T2, T3> IntoSystem<(T1, T2, T3)> for F
where
    Fut: Future<Output = ()> + Send + Sync + 'static,
    T1: SystemParameter + Clone + 'static,
    T2: SystemParameter + Clone + 'static,
    T3: SystemParameter + Clone + 'static,
{
    type System = FunctionSystem<(T1, T2, T3), Self>;

    fn into_system(self) -> Self::System {
        FunctionSystem {
            f: self,
            _phantom: PhantomData::default(),
        }
    }
}
