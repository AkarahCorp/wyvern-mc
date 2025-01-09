use tokio::sync::oneshot::Sender;
use voxidian_protocol::packet::{c2s::{config::C2SConfigPackets, handshake::C2SHandshakePackets, login::C2SLoginPackets, play::C2SPlayPackets, status::C2SStatusPackets}, s2c::{config::S2CConfigPackets, login::S2CLoginPackets, play::S2CPlayPackets, status::S2CStatusPackets}, Stage};

#[derive(Debug)]
pub enum ConnectionMessage {
    SetStage(Stage),
    GetStage(Sender<Stage>),

    SendStatusPacket(S2CStatusPackets),
    SendLoginPacket(S2CLoginPackets),
    SendConfigPacket(S2CConfigPackets),
    SendPlayPacket(S2CPlayPackets),

    ReadHandshakingPacket(Sender<Option<C2SHandshakePackets>>),
    ReadStatusPacket(Sender<Option<C2SStatusPackets>>),
    ReadLoginPacket(Sender<Option<C2SLoginPackets>>),
    ReadConfigPacket(Sender<Option<C2SConfigPackets>>),
    ReadPlayPacket(Sender<Option<C2SPlayPackets>>)
}