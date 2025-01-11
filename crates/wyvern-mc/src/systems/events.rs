use super::parameters::EventType;

#[derive(Default, Clone, Debug)]
pub struct ReceivePacketEvent<P> {
    _phantom: P
}
impl<P> EventType for ReceivePacketEvent<P> {}