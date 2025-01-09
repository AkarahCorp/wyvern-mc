use std::sync::Arc;

use tokio::sync::{mpsc::{Receiver, Sender}, oneshot::channel};
use voxidian_protocol::packet::{PacketBuf, PrefixedPacketEncode, Stage};

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

    pub async fn write_packet<P: PrefixedPacketEncode>(&self, packet: P) {
        let mut buf = PacketBuf::new();
        packet.encode_prefixed(&mut buf).unwrap();
        self.messenger.send(ConnectionMessage::SendPacket(buf)).await.unwrap();
    }
}

pub struct ConnectionWithSignal {
    pub(crate) messenger: Arc<Sender<ConnectionMessage>>,
    pub(crate) _signal: Receiver<ConnectionStoppedSignal>
}

impl ConnectionWithSignal {
    pub fn lower(&self) -> Connection {
        Connection { messenger: self.messenger.clone() }
    }
}