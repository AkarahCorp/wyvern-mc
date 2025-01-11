use crate::{player::proxy::ConnectionWithSignal, systems::typemap::TypeMap};

pub enum ServerMessage {
    SpawnConnection(ConnectionWithSignal),
    FireSystems(TypeMap)
}