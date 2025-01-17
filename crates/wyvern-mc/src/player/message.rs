use tokio::sync::oneshot::Sender;
use voxidian_protocol::packet::{PacketBuf, Stage};

use crate::{dimension::DimensionData, server::server::Server, values::key::Key};

pub enum ConnectionMessage {
    SetStage(Stage),
    GetStage(Sender<Stage>),
    SendPacket(PacketBuf),
    GetServer(Sender<Server>),
}
