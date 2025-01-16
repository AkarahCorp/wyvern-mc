use crate::{player::player::ConnectionWithSignal, systems::typemap::TypeMap};

#[derive(Debug)]
pub enum ServerMessage {
    SpawnConnection(ConnectionWithSignal),
    FireSystems(TypeMap)
}