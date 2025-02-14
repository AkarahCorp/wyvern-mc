use std::{
    collections::VecDeque,
    net::IpAddr,
    sync::{Arc, Mutex},
};

use async_net::TcpStream;
use data::PlayerData;
use flume::{Receiver, Sender};
use inventory::PlayerInventory;
use net::ConnectionStoppedSignal;
use voxidian_protocol::{
    packet::{
        PacketBuf, PacketEncode, PrefixedPacketEncode, Stage,
        processing::PacketProcessing,
        s2c::play::{
            AddEntityS2CPlayPacket, ContainerSetSlotS2CPlayPacket, ForgetLevelChunkS2CPlayPacket,
            GameEvent, GameEventS2CPlayPacket, Gamemode, PlayerPositionS2CPlayPacket,
            PlayerRotationS2CPlayPacket, RespawnS2CPlayPacket, SystemChatS2CPlayPacket,
            TeleportFlags,
        },
    },
    value::{Angle, Text, TextComponent, Uuid, VarInt},
};
use wyvern_macros::{actor, message};

use crate::{
    actors::{ActorError, ActorResult},
    dimension::Dimension,
    inventory::{Inventory, ItemStack},
    server::Server,
    values::{Vec2, Vec3},
};

pub mod chunkload;
pub mod data;
pub mod inventory;
pub mod net;
pub mod stages;

#[actor(Player, PlayerMessage)]
pub(crate) struct ConnectionData {
    pub(crate) stream: TcpStream,
    #[allow(dead_code)]
    pub(crate) addr: IpAddr,
    pub(crate) received_bytes: VecDeque<u8>,
    pub(crate) bytes_to_send: Vec<u8>,
    pub(crate) packet_processing: PacketProcessing,
    pub(crate) signal: Sender<ConnectionStoppedSignal>,
    pub(crate) connected_server: Server,
    pub(crate) stage: Arc<Mutex<Stage>>,
    pub(crate) associated_data: PlayerData,
    pub(crate) sender: Sender<PlayerMessage>,
}

#[message(Player, PlayerMessage)]
impl ConnectionData {
    #[SetStage]
    pub async fn set_stage(&mut self, stage: Stage) -> ActorResult<()> {
        *self.stage.lock().unwrap() = stage;
        Ok(())
    }

    #[GetStage]
    pub async fn get_stage(&mut self) -> ActorResult<Stage> {
        Ok(*self.stage.lock().unwrap())
    }

    #[IsLoaded]
    pub async fn is_loaded_in_world(&self) -> ActorResult<bool> {
        Ok(self.associated_data.is_loaded)
    }

    #[SendPacketBuf]
    pub async fn send_packet_buf(&mut self, buf: PacketBuf) -> ActorResult<()> {
        self.bytes_to_send.extend(buf.iter());
        Ok(())
    }

    #[GetServer]
    pub async fn get_server(&self) -> ActorResult<Server> {
        Ok(self.connected_server.clone())
    }

    #[GetDimension]
    pub async fn get_dimension(&self) -> ActorResult<Dimension> {
        self.associated_data
            .dimension
            .clone()
            .ok_or(ActorError::ActorIsNotLoaded)
    }

    #[ChangeDimension]
    pub async fn change_dimension(&mut self, dimension: Dimension) -> ActorResult<()> {
        for chunk in self.associated_data.loaded_chunks.clone() {
            self.write_packet(ForgetLevelChunkS2CPlayPacket {
                chunk_z: chunk.y(),
                chunk_x: chunk.x(),
            })
            .await;
        }

        self.associated_data.dimension = Some(dimension.clone());
        self.associated_data.loaded_chunks.clear();
        self.associated_data.last_position = Vec3::new(0.0, 0.0, 0.0);
        self.associated_data.last_direction = Vec2::new(0.0, 0.0);

        self.write_packet(RespawnS2CPlayPacket {
            dim: self
                .connected_server
                .registries()
                .await?
                .dimension_types
                .get_entry(dimension.get_dimension_type().await?)
                .unwrap(),
            dim_name: dimension.get_name().await?.into(),
            seed: 0,
            gamemode: Gamemode::Survival,
            prev_gamemode: Gamemode::None,
            is_debug: false,
            is_flat: false,
            death_loc: None,
            portal_cooldown: VarInt::from(0),
            sea_level: VarInt::from(0),
            data_kept: 0,
        })
        .await;
        self.write_packet(GameEventS2CPlayPacket {
            event: GameEvent::WaitForChunks,
            value: 0.0,
        })
        .await;
        self.write_packet(PlayerPositionS2CPlayPacket {
            teleport_id: VarInt::from(18383),
            x: 1.0,
            y: 1.0,
            z: 1.0,
            vx: 0.0,
            vy: 0.0,
            vz: 0.0,
            adyaw_deg: 0.0,
            adpitch_deg: 0.0,
            flags: TeleportFlags {
                relative_x: false,
                relative_y: false,
                relative_z: false,
                relative_pitch: true,
                relative_yaw: true,
                relative_vx: true,
                relative_vy: true,
                relative_vz: true,
                rotate_velocity: false,
            },
        })
        .await;
        self.write_packet(PlayerRotationS2CPlayPacket {
            yaw: 0.0,
            pitch: 0.0,
        })
        .await;

        for entity in dimension.get_all_entities().await? {
            let position = entity.position().await?;
            self.write_packet(AddEntityS2CPlayPacket {
                id: entity.entity_id().await?.into(),
                uuid: *entity.uuid(),
                kind: self
                    .connected_server
                    .registries()
                    .await?
                    .entity_types
                    .get_entry(entity.entity_type().await?.retype())
                    .unwrap(),
                x: position.0.x(),
                y: position.0.x(),
                z: position.0.x(),
                pitch: Angle::of_deg(position.1.x()),
                yaw: Angle::of_deg(position.1.y()),
                head_yaw: Angle::of_deg(position.1.y()),
                data: VarInt::from(0),
                vel_x: 0,
                vel_y: 0,
                vel_z: 0,
            })
            .await;
        }

        Ok(())
    }

    #[GetInvSlot]
    pub(crate) async fn get_inv_slot(&self, slot: usize) -> ActorResult<ItemStack> {
        self.associated_data.inventory.get_slot(slot).await
    }

    #[SetInvSlot]
    pub(crate) async fn set_inv_slot(&mut self, slot: usize, item: ItemStack) -> ActorResult<()> {
        let copy = item.clone();

        let slot = self
            .associated_data
            .screen
            .map(|x| x.container_slot_count())
            .unwrap_or(0)
            + slot;
        self.associated_data.inventory.set_slot(slot, copy).await?;

        let packet = ContainerSetSlotS2CPlayPacket {
            window_id: VarInt::new(self.associated_data.window_id.unwrap_or(0) as i32),
            state_id: VarInt::new(1),
            slot: slot as u16,
            slot_data: item.into(),
        };
        self.write_packet(packet).await;
        Ok(())
    }

    #[ReadData]
    pub(crate) async fn read_data(
        &self,
        f: Box<dyn Fn(&PlayerData) + Send + Sync>,
    ) -> ActorResult<()> {
        f(&self.associated_data);
        Ok(())
    }
}

impl Player {
    pub async fn write_packet<P: PrefixedPacketEncode + std::fmt::Debug>(
        &self,
        packet: P,
    ) -> ActorResult<()> {
        let mut buf = PacketBuf::new();
        packet.encode_prefixed(&mut buf).unwrap();

        let mut len_buf: PacketBuf = PacketBuf::new();
        VarInt::from(buf.iter().count())
            .encode(&mut len_buf)
            .unwrap();

        let mut new_buf = PacketBuf::new();
        new_buf.write_u8s(len_buf.as_slice());
        new_buf.write_u8s(buf.as_slice());
        self.send_packet_buf(new_buf).await?;

        Ok(())
    }

    pub fn get_inventory(&self) -> ActorResult<PlayerInventory> {
        Ok(PlayerInventory {
            player: self.clone(),
        })
    }

    pub async fn send_message(&self, content: &str) -> ActorResult<()> {
        let mut text = Text::new();
        text.push(TextComponent::of_literal(content));
        self.write_packet(SystemChatS2CPlayPacket {
            content: text.to_nbt(),
            is_actionbar: false,
        })
        .await?;
        Ok(())
    }

    pub async fn send_action_bar(&self, content: &str) -> ActorResult<()> {
        let mut text = Text::new();
        text.push(TextComponent::of_literal(content));
        self.write_packet(SystemChatS2CPlayPacket {
            content: text.to_nbt(),
            is_actionbar: true,
        })
        .await?;
        Ok(())
    }

    pub async fn username(&self) -> ActorResult<String> {
        let value = Arc::new(Mutex::new(String::new()));
        let value_clone = value.clone();
        self.read_data(Box::new(move |data| {
            *value_clone.lock().unwrap() = data.username.clone();
        }))
        .await?;
        Ok(value.lock().unwrap().clone())
    }

    pub async fn uuid(&self) -> ActorResult<Uuid> {
        let value = Arc::new(Mutex::new(Uuid::nil()));
        let value_clone = value.clone();
        self.read_data(Box::new(move |data| {
            *value_clone.lock().unwrap() = data.uuid;
        }))
        .await?;
        Ok(*value.lock().unwrap())
    }

    pub async fn position(&self) -> ActorResult<(Vec3<f64>, Vec2<f32>)> {
        let value = Arc::new(Mutex::new((Vec3::new(0.0, 0.0, 0.0), Vec2::new(0.0, 0.0))));
        let value_clone = value.clone();
        self.read_data(Box::new(move |data| {
            *value_clone.lock().unwrap() = (data.last_position, data.last_direction);
        }))
        .await?;
        Ok(*value.lock().unwrap())
    }
}

impl ConnectionData {
    pub async fn write_packet<P: PrefixedPacketEncode + std::fmt::Debug>(&mut self, packet: P) {
        let mut buf = PacketBuf::new();
        packet.encode_prefixed(&mut buf).unwrap();

        let mut len_buf = PacketBuf::new();
        VarInt::from(buf.iter().count())
            .encode(&mut len_buf)
            .unwrap();

        self.bytes_to_send.extend(len_buf);
        self.bytes_to_send.extend(buf);
    }
}

#[derive(Debug)]
pub struct ConnectionWithSignal {
    pub(crate) player: Player,
    pub(crate) _signal: Receiver<ConnectionStoppedSignal>,
    pub(crate) stage: Arc<Mutex<Stage>>,
}

impl ConnectionWithSignal {
    pub fn lower(&self) -> Player {
        self.player.clone()
    }
}
