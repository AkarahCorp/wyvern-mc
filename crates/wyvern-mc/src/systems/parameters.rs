use super::typemap::TypeMap;

pub trait SystemParameter {
    fn query(resources: &TypeMap) -> Option<Self> where Self: Sized;
}

#[derive(Clone)]
pub struct Event<T: EventType + Clone> {
    data: T
}

impl<T: EventType + Clone> Event<T> {
    pub fn new(data: T) -> Self {
        Event { data }
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
}

impl<T: EventType + Clone + 'static> SystemParameter for Event<T> {
    fn query(resources: &TypeMap) -> Option<Self> {
        resources.get::<Self>().cloned()
    }
}

impl<T: Clone + 'static> SystemParameter for Param<T> {
    fn query(resources: &TypeMap) -> Option<Self> {
        resources.get::<Self>().cloned()
    }
}

impl<T1, T2> SystemParameter for (T1, T2)
where
    T1: SystemParameter + Clone + 'static,
    T2: SystemParameter + Clone + 'static {
    fn query(resources: &TypeMap) -> Option<Self> {
        Some((T1::query(resources)?, T2::query(resources)?))
    }
}

impl<T1, T2, T3> SystemParameter for (T1, T2, T3)
where
    T1: SystemParameter + Clone + 'static,
    T2: SystemParameter + Clone + 'static,
    T3: SystemParameter + Clone + 'static {
    fn query(resources: &TypeMap) -> Option<Self> {
        Some((T1::query(resources)?, T2::query(resources)?, T3::query(resources)?))
    }
}