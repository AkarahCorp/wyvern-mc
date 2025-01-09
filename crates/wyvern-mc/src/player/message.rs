use tokio::sync::oneshot::Sender;
use voxidian_protocol::packet::{PacketBuf, Stage};

#[derive(Debug)]
pub enum ConnectionMessage {
    SetStage(Stage),
    GetStage(Sender<Stage>),
    SendPacket(PacketBuf)
}