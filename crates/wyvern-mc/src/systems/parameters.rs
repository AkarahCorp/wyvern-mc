use std::{fmt::Debug, marker::PhantomData, ops::{Deref, DerefMut}, sync::Arc};

use crate::server::proxy::Server;

use super::typemap::TypeMap;

pub trait SystemParameter: Clone + 'static {
    fn query(resources: &TypeMap, server: &Server) -> Option<Self> where Self: Sized;
}






pub struct Event<T: EventType> {
    _data: PhantomData<T>
}

impl<T: EventType> Event<T> {
    pub fn new() -> Self {
        Event { _data: PhantomData::default() }
    }
}

impl<T: EventType> Clone for Event<T> {
    fn clone(&self) -> Self {
        Self { _data: PhantomData::default() }
    }
} 

pub trait EventType {}




#[derive(Debug)]
pub struct Param<T: Debug> {
    data: Arc<T>
}

impl<T: Debug> Param<T> {
    pub fn new(data: T) -> Self {
        Param { data: Arc::new(data) }
    }

    pub fn into_inner(self) -> Arc<T> {
        self.data
    }
}

impl<T: Debug> Deref for Param<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<T: Debug> Clone for Param<T> {
    fn clone(&self) -> Self {
        Self { data: self.data.clone() }
    }
}



#[derive(Clone, Debug)]
pub struct Query<T: Clone + Debug> {
    data: T
}

impl<T: Clone + Debug> Query<T> {
    pub fn new(data: T) -> Self {
        Query { data }
    }

    pub fn into_inner(self) -> T {
        self.data
    }
}

impl<T: Clone + Debug> Deref for Query<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<T: Clone + Debug> DerefMut for Query<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}






impl<T: EventType + 'static + Send + Sync> SystemParameter for Event<T> {
    fn query(resources: &TypeMap, _server: &Server) -> Option<Self> {
        resources.get::<Self>().map(|_| Event { _data: PhantomData::default() })
    }
}

impl<T: 'static + Send + Sync + Debug> SystemParameter for Param<T> {
    fn query(resources: &TypeMap, _server: &Server) -> Option<Self> {
        let out = resources.get::<Self>().cloned();
        println!("querying parameter: {:?}", out);
        out
    }
}

impl<T1> SystemParameter for (T1,)
where
    T1: SystemParameter + Clone + 'static {
    fn query(resources: &TypeMap, server: &Server) -> Option<Self> {
        Some((T1::query(resources, server)?,))
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

impl SystemParameter for () {
    fn query(_resources: &TypeMap, _server: &Server) -> Option<Self> {
        Some(())
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

impl<T1, T2, T3, T4> SystemParameter for (T1, T2, T3, T4)
where
    T1: SystemParameter + Clone + 'static,
    T2: SystemParameter + Clone + 'static,
    T3: SystemParameter + Clone + 'static,
    T4: SystemParameter + Clone + 'static {
    fn query(resources: &TypeMap, server: &Server) -> Option<Self> {
        Some((T1::query(resources, server)?, T2::query(resources, server)?, T3::query(resources, server)?, T4::query(resources, server)?))
    }
}