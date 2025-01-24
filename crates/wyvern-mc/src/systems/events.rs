use std::marker::PhantomData;

use super::parameters::EventType;

#[derive(Default, Clone, Debug)]
pub struct ReceivePacketEvent<P> {
    _phantom: PhantomData<P>,
}
impl<P> EventType for ReceivePacketEvent<P> {}

#[derive(Default, Clone, Debug)]
pub struct ServerTickEvent;
impl EventType for ServerTickEvent {}
