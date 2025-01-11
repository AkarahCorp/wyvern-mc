use crate::server::proxy::Server;

use super::{function::FunctionSystem, parameters::SystemParameter, typemap::TypeMap, BoxedFuture};

pub trait System {
    fn run(&mut self, resources: &mut TypeMap, server: Server) -> BoxedFuture;
}

impl<Fut, F: FnMut() -> Fut> System for FunctionSystem<(), F>
where
    Fut: Future<Output = ()> + Send + Sync + 'static, {
    fn run(&mut self, _resources: &mut TypeMap, _server: Server) -> BoxedFuture {
        Box::pin((self.f)())
    }
}

impl<Fut, F, T1> System for FunctionSystem<(T1,), F>
where
    Fut: Future<Output = ()> + Send + Sync + 'static,
    F: FnMut(T1) -> Fut,
    
    T1: SystemParameter + Clone + 'static {
    fn run(&mut self, resources: &mut TypeMap, server: Server) -> BoxedFuture {
        Box::pin((self.f)(T1::query(&resources, &server).unwrap().clone()))
    }
}