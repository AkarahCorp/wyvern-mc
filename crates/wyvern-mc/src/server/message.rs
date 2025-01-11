use crate::player::proxy::ConnectionWithSignal;

pub enum ServerMessage {
    SpawnConnection(ConnectionWithSignal)
}