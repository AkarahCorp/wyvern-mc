use voxidian_protocol::{packet::{c2s::{login::C2SLoginPackets, status::C2SStatusPackets}, s2c::{login::LoginFinishedS2CLoginPacket, status::{PongResponseS2CStatusPacket, StatusResponse, StatusResponsePlayers, StatusResponseVersion}}, PacketBuf, PacketEncode, PrefixedPacketEncode, Stage}, value::{LengthPrefixHashMap, Text, VarInt}};

use super::{message::ConnectionMessage, net::ConnectionData};

impl ConnectionData {
    pub fn write_packet<P: PrefixedPacketEncode>(&mut self, packet: P) {

        let mut buf = PacketBuf::new();
        packet.encode_prefixed(&mut buf).unwrap();

        
        let mut len_buf = PacketBuf::new();
        VarInt::from(buf.iter().count()).encode(&mut len_buf).unwrap();

        let snd = self.sender.clone();
        tokio::spawn(async move { 
            snd.send(ConnectionMessage::SendPacket(len_buf)).await.unwrap(); 
            snd.send(ConnectionMessage::SendPacket(buf)).await.unwrap(); 
        });
    }

    pub fn status_stage(&mut self) {
        self.read_packets(|packet: C2SStatusPackets, this| {
            match packet {
                C2SStatusPackets::StatusRequest(_packet) => {
                    this.write_packet(
                        StatusResponse {
                            version: StatusResponseVersion {
                                name: "1.21.4".to_string(),
                                protocol: 769,
                            },
                            players: StatusResponsePlayers {
                                online: 0,
                                max: 100,
                                sample: vec![],
                            },
                            desc: Text::new(),
                            favicon_png_b64: "".to_string(),
                            enforce_chat_reports: false,
                            prevent_chat_reports: true
                        }.to_packet()
                    );
                },
                C2SStatusPackets::PingRequest(packet) => {
                    this.write_packet(PongResponseS2CStatusPacket { timestamp: packet.timestamp });
                },
            }
        });
    }

    pub fn login_stage(&mut self) {
        self.read_packets(|packet: C2SLoginPackets, this| {
            match packet {
                C2SLoginPackets::CustomQueryAnswer(_packet) => todo!(),
                C2SLoginPackets::LoginAcknowledged(_packet) => {
                    this.stage = Stage::Config;
                },
                C2SLoginPackets::Key(_packet) => todo!(),
                C2SLoginPackets::Hello(packet) => {
                    this.associated_data.username = packet.username.clone();
                    this.associated_data.uuid = packet.uuid.clone();
                    this.write_packet(LoginFinishedS2CLoginPacket {
                        uuid: packet.uuid,
                        username: packet.username,
                        props: LengthPrefixHashMap::new(),
                    });
                },
                C2SLoginPackets::CookieResponse(_packet) => todo!(),
            }
        });
    }
}