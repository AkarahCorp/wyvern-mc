use core::panic;
use std::{
    collections::VecDeque,
    fmt::Debug,
    io::{ErrorKind, Read, Write},
    net::{IpAddr, TcpStream},
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use crate::{
    actors::{Actor, ActorResult},
    components::DataComponentMap,
    runtime::Runtime,
};
use flume::{Receiver, Sender};
use voxidian_protocol::packet::{
    DecodeError, PrefixedPacketDecode, Stage,
    c2s::handshake::C2SHandshakePackets,
    processing::PacketProcessing,
    s2c::play::{Gamemode, KeepAliveS2CPlayPacket},
};

use crate::{player::PlayerMessage, server::Server};

use super::{ConnectionData, ConnectionWithSignal, Player, PlayerComponents, data::PlayerData};

pub struct ConnectionStoppedSignal;

impl ConnectionData {
    pub fn connection_channel(
        stream: TcpStream,
        addr: IpAddr,
        server: Server,
        stage: Arc<Mutex<Stage>>,
    ) -> ConnectionWithSignal {
        let (signal_tx, signal_rx) = flume::bounded(1);
        let (data_tx, data_rx) = flume::unbounded();

        let stage2 = stage.clone();
        let data_tx2 = data_tx.clone();
        Runtime::spawn_actor(move || {
            ConnectionData::new_conn(stream, addr, data_tx2, data_rx, signal_tx, server, stage2)
        });

        ConnectionWithSignal {
            player: Player { sender: data_tx },
            _signal: signal_rx,
            stage,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn new_conn(
        stream: TcpStream,
        addr: IpAddr,
        sender: Sender<PlayerMessage>,
        receiver: Receiver<PlayerMessage>,
        signal: Sender<ConnectionStoppedSignal>,
        server: Server,
        stage: Arc<Mutex<Stage>>,
    ) {
        stream.set_nonblocking(true).unwrap();
        let conn = ConnectionData {
            stream,
            addr,
            received_bytes: VecDeque::new(),
            bytes_to_send: Vec::new(),
            packet_processing: PacketProcessing::NONE,
            receiver,
            sender: sender.downgrade(),
            signal,
            stage,
            connected_server: server,
            associated_data: PlayerData::default(),
            mojauth: None,
            components: DataComponentMap::new()
                .with(PlayerComponents::GAMEMODE, Gamemode::Survival),
            last_saved_components: DataComponentMap::new(),
        };

        conn.event_loop();
    }

    pub fn event_loop(mut self) {
        loop {
            let result = self.handle_incoming_bytes();
            if result.is_err() {
                log::info!("A player has disconnected. Stopping their connection data...");

                if let Some(dim) = &self.associated_data.dimension {
                    let _ = dim.remove_entity(self.associated_data.uuid);
                }
                self.signal.send(ConnectionStoppedSignal).unwrap();
                drop(self);
                return;
            }
            self.handle_messages();
            let _ = self.read_incoming_packets();
            self.write_outgoing_packets();

            let now = Instant::now();
            if now > self.associated_data.last_sent_keep_alive + Duration::from_secs(5)
                && *self.stage.lock().unwrap() == Stage::Play
            {
                self.write_packet(KeepAliveS2CPlayPacket(10));
                self.associated_data.last_sent_keep_alive = Instant::now();
            }
        }
    }

    pub fn handle_incoming_bytes(&mut self) -> Result<(), ()> {
        let mut buf = [0; 512];

        match self.stream.read(&mut buf) {
            Ok(bytes_read) => {
                if bytes_read == 0 {
                    return Err(());
                }
                for byte in &buf[0..bytes_read] {
                    let byte = self
                        .packet_processing
                        .secret_cipher
                        .decrypt_u8(*byte)
                        .unwrap();
                    self.received_bytes.push_back(byte);
                }
            }
            Err(e) if e.kind() == ErrorKind::WouldBlock => {}
            Err(_) => return Err(()),
        };

        Ok(())
    }

    pub fn read_incoming_packets(&mut self) -> ActorResult<()> {
        let stage = *self.stage.lock().unwrap();
        match stage {
            Stage::Handshake => {
                self.read_packets(|packet: C2SHandshakePackets, this: &mut Self| {
                    let C2SHandshakePackets::Intention(packet) = packet;
                    *this.stage.lock().unwrap() = packet.intended_stage.into_stage();
                    Ok(())
                })?;
            }
            Stage::Status => {
                self.status_stage()?;
            }
            Stage::Login => {
                self.login_stage()?;
            }
            Stage::Config => {
                self.configuration_stage()?;
            }
            Stage::Play => {
                self.play_phase()?;
            }
            Stage::Transfer => todo!("doesn't exist, this needs to be removed D:"),
        }
        Ok(())
    }

    pub fn write_outgoing_packets(&mut self) {
        loop {
            if self.bytes_to_send.is_empty() {
                break;
            }

            match self.stream.write_all(&self.bytes_to_send) {
                Ok(()) => {}
                Err(e) if e.kind() == ErrorKind::WouldBlock => {}
                Err(e) => panic!("{:?}", e),
            }
            self.bytes_to_send.clear();
        }
    }

    pub fn read_packets<
        T: PrefixedPacketDecode + Debug,
        F: FnOnce(T, &mut Self) -> ActorResult<()>,
    >(
        &mut self,
        f: F,
    ) -> ActorResult<()> {
        match self
            .packet_processing
            .decode_from_raw_queue(self.received_bytes.iter().copied())
        {
            Ok((mut buf, consumed)) => {
                if consumed == 0 {
                    return Ok(());
                }

                let byte_cache = self.received_bytes.clone();
                let mut rv = Vec::with_capacity(consumed);
                for _ in 0..consumed {
                    rv.push(self.received_bytes.pop_front().unwrap());
                }

                let buf_copy = buf.clone();
                match T::decode_prefixed(&mut buf) {
                    Ok(packet) => {
                        f(packet, self)?;
                        Ok(())
                    }
                    Err(DecodeError::EndOfBuffer) => {
                        log::error!(
                            "The server has encountered a packet decoding error!

                        Buffer received: {:?}
                        Buffer after receiving: {:?}
                        Received bytes before consuming: {:?}
                        Bytes left to receive: {:?}
                        Bytes consumed: {:?}
                        ",
                            buf_copy.iter().collect::<Vec<u8>>(),
                            buf.iter().collect::<Vec<u8>>(),
                            byte_cache,
                            self.received_bytes,
                            rv
                        );
                        Ok(())
                    }
                    Err(e) => {
                        panic!("{:?}", e);
                    }
                }
            }
            Err(DecodeError::EndOfBuffer) => Ok(()),
            Err(e) => {
                panic!("err: {:?}", e);
            }
        }
    }
}
