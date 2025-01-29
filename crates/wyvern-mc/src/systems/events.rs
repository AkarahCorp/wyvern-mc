use std::marker::PhantomData;

use super::parameters::EventType;

#[derive(Default, Clone, Debug)]
pub struct ReceivePacketEvent<P> {
    _phantom: PhantomData<P>,
}
impl<P> EventType for ReceivePacketEvent<P> {}

macro_rules! make_events {
    ( $($name:ident)*) => {
        $(#[derive(Default, Clone, Debug)]
        pub struct $name;
        impl EventType for $name {})*
    };
}

make_events! {
    ServerTickEvent
    PlayerMoveEvent
    ChunkLoadEvent
    DimensionCreateEvent
}
