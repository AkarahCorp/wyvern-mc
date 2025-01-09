use std::sync::Arc;

use tokio::sync::{mpsc::{Receiver, Sender}, oneshot::channel};
use voxidian_protocol::packet::{c2s::{config::C2SConfigPackets, handshake::C2SHandshakePackets, login::C2SLoginPackets, play::C2SPlayPackets, status::C2SStatusPackets}, Stage};

use super::{message::ConnectionMessage, net::ConnectionStoppedSignal};

#[derive(Clone)]
pub struct Connection {
    pub(crate) messenger: Arc<Sender<ConnectionMessage>>,
}

impl Connection {
    pub async fn get_stage(&self) -> Stage {
        let (tx, rx) = channel();
        self.messenger.send(ConnectionMessage::GetStage(tx)).await.unwrap();
        rx.await.unwrap()
    }

    pub async fn set_stage(&self, stage: Stage) {
        self.messenger.send(ConnectionMessage::SetStage(stage)).await.unwrap();
    }

    pub async fn read_handshaking_packet(&self) -> Option<C2SHandshakePackets> {
        let (tx, rx) = channel();
        self.messenger.send(ConnectionMessage::ReadHandshakingPacket(tx)).await.unwrap();
        rx.await.unwrap()
    }

    pub async fn read_status_packet(&self) -> Option<C2SStatusPackets> {
        let (tx, rx) = channel();
        self.messenger.send(ConnectionMessage::ReadStatusPacket(tx)).await.unwrap();
        rx.await.unwrap()
    }

    pub async fn read_login_packet(&self) -> Option<C2SLoginPackets> {
        let (tx, rx) = channel();
        self.messenger.send(ConnectionMessage::ReadLoginPacket(tx)).await.unwrap();
        rx.await.unwrap()
    }

    pub async fn read_config_packet(&self) -> Option<C2SConfigPackets> {
        let (tx, rx) = channel();
        self.messenger.send(ConnectionMessage::ReadConfigPacket(tx)).await.unwrap();
        rx.await.unwrap()
    }

    pub async fn read_play_packet(&self) -> Option<C2SPlayPackets> {
        let (tx, rx) = channel();
        self.messenger.send(ConnectionMessage::ReadPlayPacket(tx)).await.unwrap();
        rx.await.unwrap()
    }
}

pub struct ConnectionWithSignal {
    pub(crate) messenger: Arc<Sender<ConnectionMessage>>,
    pub(crate) signal: Receiver<ConnectionStoppedSignal>
}

impl ConnectionWithSignal {
    pub fn lower(&self) -> Connection {
        Connection { messenger: self.messenger.clone() }
    }
}