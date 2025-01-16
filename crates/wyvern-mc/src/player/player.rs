use std::sync::Arc;

use tokio::sync::{mpsc::{Receiver, Sender}, oneshot::channel};
use voxidian_protocol::{packet::{PacketBuf, PacketEncode, PrefixedPacketEncode, Stage}, value::VarInt};

use crate::server::server::Server;

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

        let mut len_buf = PacketBuf::new();
        VarInt::from(buf.iter().count()).encode(&mut len_buf).unwrap();

        let _ = self.messenger.send(ConnectionMessage::SendPacket(len_buf)).await;
        let _ = self.messenger.send(ConnectionMessage::SendPacket(buf)).await;
    }

    pub async fn get_server(&self) -> Server {
        let (tx, rx) = channel();
        let _ = self.messenger.send(ConnectionMessage::GetServer(tx)).await;
        rx.await.unwrap()
    }
}

#[derive(Debug)]
pub struct ConnectionWithSignal {
    pub(crate) messenger: Arc<Sender<ConnectionMessage>>,
    pub(crate) _signal: Receiver<ConnectionStoppedSignal>
}

impl ConnectionWithSignal {
    pub fn lower(&self) -> Player {
        Player { messenger: self.messenger.clone() }
    }
}