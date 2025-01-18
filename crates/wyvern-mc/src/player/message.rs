use tokio::sync::oneshot::Sender;
use voxidian_protocol::packet::{PacketBuf, Stage};

use crate::server::server::Server;

pub enum ConnectionMessage {
    SetStage(Stage),
    GetStage(Sender<Stage>),
    SendPacket(PacketBuf),
    GetServer(Sender<Server>),
}
