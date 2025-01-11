use std::sync::Arc;

use tokio::sync::{mpsc::{Receiver, Sender}, oneshot::channel};
use voxidian_protocol::packet::{PacketBuf, PrefixedPacketEncode, Stage};

use crate::server::proxy::Server;

use super::{message::ConnectionMessage, net::ConnectionStoppedSignal};

#[derive(Clone)]
pub struct Player {
    pub(crate) messenger: Arc<Sender<ConnectionMessage>>,
}

impl Player {
    pub async fn get_stage(&self) -> Stage {
        let (tx, rx) = channel();
        let _ = self.messenger.send(ConnectionMessage::GetStage(tx)).await;
        rx.await.unwrap()
    }

    pub async fn set_stage(&self, stage: Stage) {
        let _ = self.messenger.send(ConnectionMessage::SetStage(stage)).await;
    }

    pub async fn write_packet<P: PrefixedPacketEncode>(&self, packet: P) {
        let mut buf = PacketBuf::new();
        packet.encode_prefixed(&mut buf).unwrap();
        let _ = self.messenger.send(ConnectionMessage::SendPacket(buf)).await;
    }

    pub async fn get_server(&self) -> Server {
        let (tx, rx) = channel();
        let _ = self.messenger.send(ConnectionMessage::GetServer(tx)).await;
        rx.await.unwrap()
    }
}

pub struct ConnectionWithSignal {
    pub(crate) messenger: Arc<Sender<ConnectionMessage>>,
    pub(crate) _signal: Receiver<ConnectionStoppedSignal>
}

impl ConnectionWithSignal {
    pub fn lower(&self) -> Player {
        Player { messenger: self.messenger.clone() }
    }
}