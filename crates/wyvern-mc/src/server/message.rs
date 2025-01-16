use crate::{player::proxy::ConnectionWithSignal, systems::typemap::TypeMap};

#[derive(Debug)]
pub enum ServerMessage {
    SpawnConnection(ConnectionWithSignal),
    FireSystems(TypeMap)
}