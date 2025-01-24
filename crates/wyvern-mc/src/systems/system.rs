use crate::server::Server;

use super::{BoxedFuture, function::FunctionSystem, parameters::SystemParameter, typemap::TypeMap};

pub trait System {
    fn run(&mut self, resources: &mut TypeMap, server: Server) -> Option<BoxedFuture>;
}

impl<Fut, F: FnMut() -> Fut> System for FunctionSystem<(), F>
where
    Fut: Future<Output = ()> + Send + Sync + 'static,
{
    fn run(&mut self, _resources: &mut TypeMap, _server: Server) -> Option<BoxedFuture> {
        Some(Box::pin((self.f)()))
    }
}

impl<Fut, F, T1> System for FunctionSystem<(T1,), F>
where
    Fut: Future<Output = ()> + Send + Sync + 'static,
    F: FnMut(T1) -> Fut,
    T1: SystemParameter + Clone + 'static,
{
    fn run(&mut self, resources: &mut TypeMap, server: Server) -> Option<BoxedFuture> {
        Some(Box::pin((self.f)(T1::query(resources, &server)?.clone())))
    }
}

impl<Fut, F, T1, T2> System for FunctionSystem<(T1, T2), F>
where
    Fut: Future<Output = ()> + Send + Sync + 'static,
    F: FnMut(T1, T2) -> Fut,
    T1: SystemParameter + Clone + 'static,
    T2: SystemParameter + Clone + 'static,
{
    fn run(&mut self, resources: &mut TypeMap, server: Server) -> Option<BoxedFuture> {
        Some(Box::pin((self.f)(
            T1::query(resources, &server)?.clone(),
            T2::query(resources, &server)?.clone(),
        )))
    }
}

impl<Fut, F, T1, T2, T3> System for FunctionSystem<(T1, T2, T3), F>
where
    Fut: Future<Output = ()> + Send + Sync + 'static,
    F: FnMut(T1, T2, T3) -> Fut,
    T1: SystemParameter + Clone + 'static,
    T2: SystemParameter + Clone + 'static,
    T3: SystemParameter + Clone + 'static,
{
    fn run(&mut self, resources: &mut TypeMap, server: Server) -> Option<BoxedFuture> {
        Some(Box::pin((self.f)(
            T1::query(resources, &server)?.clone(),
            T2::query(resources, &server)?.clone(),
            T3::query(resources, &server)?.clone(),
        )))
    }
}
