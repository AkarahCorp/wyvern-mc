use voxidian_protocol::{
    packet::{
        c2s::status::C2SStatusPackets,
        s2c::status::{
            PongResponseS2CStatusPacket, StatusResponse, StatusResponsePlayers,
            StatusResponseVersion,
        },
    },
    value::Text,
};

use crate::{actors::ActorResult, player::ConnectionData};

impl ConnectionData {
    pub async fn status_stage(&mut self) -> ActorResult<()> {
        self.read_packets(async |packet: C2SStatusPackets, this| {
            log::debug!("Packet: {:?}", packet);
            match packet {
                C2SStatusPackets::StatusRequest(_packet) => {
                    this.write_packet(
                        StatusResponse {
                            version: StatusResponseVersion {
                                name: "1.21.4".to_string(),
                                protocol: 769,
                            },
                            players: Some(StatusResponsePlayers {
                                online: 0,
                                max: 100,
                                sample: vec![],
                            }),
                            desc: Text::new(),
                            favicon_png_b64: "".to_string(),
                            enforce_chat_reports: false,
                            prevent_chat_reports: true,
                        }
                        .to_packet(),
                    )
                    .await;
                }
                C2SStatusPackets::PingRequest(packet) => {
                    this.write_packet(PongResponseS2CStatusPacket {
                        timestamp: packet.timestamp,
                    })
                    .await;
                }
            }

            Ok(())
        })
        .await
    }
}
