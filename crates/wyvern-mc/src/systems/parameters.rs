use std::ops::{Deref, DerefMut};

use crate::server::proxy::Server;

use super::typemap::TypeMap;

pub trait SystemParameter {
    fn query(resources: &TypeMap, server: &Server) -> Option<Self> where Self: Sized;
}






#[derive(Clone)]
pub struct Event<T: EventType + Clone> {
    _data: T
}

impl<T: EventType + Clone> Event<T> {
    pub fn new(data: T) -> Self {
        Event { _data: data }
    }
}

pub trait EventType {}






#[derive(Clone)]
pub struct Param<T: Clone> {
    data: T
}

impl<T: Clone> Param<T> {
    pub fn new(data: T) -> Self {
        Param { data }
    }

    pub fn into_inner(self) -> T {
        self.data
    }
}

impl<T: Clone> Deref for Param<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<T: Clone> DerefMut for Param<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}





#[derive(Clone)]
pub struct Query<T: Clone> {
    data: T
}

impl<T: Clone> Query<T> {
    pub fn new(data: T) -> Self {
        Query { data }
    }

    pub fn into_inner(self) -> T {
        self.data
    }
}

impl<T: Clone> Deref for Query<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<T: Clone> DerefMut for Query<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}






impl<T: EventType + Clone + 'static> SystemParameter for Event<T> {
    fn query(resources: &TypeMap, _server: &Server) -> Option<Self> {
        resources.get::<Self>().cloned()
    }
}

impl<T: Clone + 'static> SystemParameter for Param<T> {
    fn query(resources: &TypeMap, _server: &Server) -> Option<Self> {
        resources.get::<Self>().cloned()
    }
}

impl<T1, T2> SystemParameter for (T1, T2)
where
    T1: SystemParameter + Clone + 'static,
    T2: SystemParameter + Clone + 'static {
    fn query(resources: &TypeMap, server: &Server) -> Option<Self> {
        Some((T1::query(resources, server)?, T2::query(resources, server)?))
    }
}

impl<T1, T2, T3> SystemParameter for (T1, T2, T3)
where
    T1: SystemParameter + Clone + 'static,
    T2: SystemParameter + Clone + 'static,
    T3: SystemParameter + Clone + 'static {
    fn query(resources: &TypeMap, server: &Server) -> Option<Self> {
        Some((T1::query(resources, server)?, T2::query(resources, server)?, T3::query(resources, server)?))
    }
}