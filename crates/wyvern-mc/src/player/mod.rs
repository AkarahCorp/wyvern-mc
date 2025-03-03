use std::{
    collections::VecDeque,
    net::{IpAddr, TcpStream},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
};

use data::PlayerData;
use flume::{Receiver, Sender, WeakSender};
use inventory::PlayerInventory;
use net::ConnectionStoppedSignal;
use voxidian_protocol::{
    mojang::auth_verify::MojAuthProperty,
    packet::{
        PacketBuf, PacketEncode, PrefixedPacketEncode, Stage,
        processing::{PacketProcessing, PrivateKey, PublicKey},
        s2c::play::{
            AddEntityS2CPlayPacket, ContainerSetSlotS2CPlayPacket, ForgetLevelChunkS2CPlayPacket,
            GameEvent, GameEventS2CPlayPacket, Gamemode, OpenScreenS2CPlayPacket,
            PlayerPositionS2CPlayPacket, PlayerRotationS2CPlayPacket, RespawnDataKept,
            RespawnS2CPlayPacket, ScreenWindowKind, SoundCategory, SoundEntityS2CPlayPacket,
            SystemChatS2CPlayPacket, TeleportFlags,
        },
    },
    registry::RegEntry,
    value::{Angle, ProfileProperty, Text as PtcText, TextComponent, Uuid, VarInt},
};
use wyvern_macros::{actor, message};

use crate::{
    actors::{ActorError, ActorResult},
    dimension::Dimension,
    inventory::{DataInventory, Inventory, ItemStack},
    runtime::Runtime,
    server::Server,
    values::{Sound, Text, Vec2, Vec3},
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
    pub(crate) sender: WeakSender<PlayerMessage>,
    pub(crate) public_key: Option<PublicKey>,
    pub(crate) private_key: Option<PrivateKey>,
    pub(crate) verify_token: Vec<u8>,
    pub(crate) props: Vec<MojAuthProperty>,
    pub(crate) is_loaded: Arc<AtomicBool>,
}

#[message(Player, PlayerMessage)]
impl ConnectionData {
    #[SetStage]
    pub fn set_stage(&mut self, stage: Stage) -> ActorResult<()> {
        *self.stage.lock().unwrap() = stage;
        Ok(())
    }

    #[GetStage]
    pub fn stage(&mut self) -> ActorResult<Stage> {
        Ok(*self.stage.lock().unwrap())
    }

    #[IsLoaded]
    pub fn is_loaded(&self) -> ActorResult<bool> {
        Ok(self.is_loaded.load(Ordering::Acquire))
    }

    #[SendPacketBuf]
    pub(crate) fn send_packet_buf(&mut self, buf: PacketBuf) -> ActorResult<()> {
        let cipherdata = self.packet_processing.encode_encrypt(buf).unwrap();
        self.bytes_to_send.extend(cipherdata.as_slice());
        Ok(())
    }

    #[GetServer]
    pub fn server(&self) -> ActorResult<Server> {
        Ok(self.connected_server.clone())
    }

    #[GetDimension]
    pub fn dimension(&self) -> ActorResult<Dimension> {
        self.associated_data
            .dimension
            .clone()
            .ok_or(ActorError::ActorIsNotLoaded)
    }

    #[MojAuthProps]
    pub fn auth_props(&self) -> ActorResult<Vec<ProfileProperty>> {
        Ok(self
            .props
            .iter()
            .map(|x| ProfileProperty {
                name: x.name.clone(),
                value: x.value.clone(),
                sig: Some(x.sig.clone()),
            })
            .collect::<Vec<_>>())
    }

    #[ChangeDimension]
    pub fn set_dimension(&mut self, dimension: Dimension) -> ActorResult<()> {
        for chunk in self.associated_data.loaded_chunks.clone() {
            self.write_packet(ForgetLevelChunkS2CPlayPacket {
                chunk_z: chunk.y(),
                chunk_x: chunk.x(),
            });
        }

        self.associated_data.dimension = Some(dimension.clone());
        self.associated_data.loaded_chunks.clear();
        self.associated_data.last_position = Vec3::new(0.0, 0.0, 0.0);
        self.associated_data.last_direction = Vec2::new(0.0, 0.0);

        self.write_packet(RespawnS2CPlayPacket {
            dim: unsafe {
                RegEntry::new_unchecked(
                    self.connected_server
                        .registries()?
                        .dimension_types
                        .get_entry(dimension.dimension_type()?)
                        .unwrap()
                        .id(),
                )
            },
            dim_name: dimension.name()?.into(),
            seed: 0,
            gamemode: Gamemode::Survival,
            prev_gamemode: Gamemode::None,
            is_debug: false,
            is_flat: false,
            death_loc: None,
            portal_cooldown: VarInt::from(0),
            sea_level: VarInt::from(0),
            data_kept: RespawnDataKept {
                keep_attributes: true,
                keep_metadata: true,
            },
        });
        self.write_packet(GameEventS2CPlayPacket {
            event: GameEvent::WaitForChunks,
            value: 0.0,
        });
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
        });
        self.write_packet(PlayerRotationS2CPlayPacket {
            yaw: 0.0,
            pitch: 0.0,
        });

        for entity in dimension.entities()? {
            let position = entity.position()?;
            self.write_packet(AddEntityS2CPlayPacket {
                id: entity.entity_id()?.into(),
                uuid: *entity.uuid(),
                kind: self
                    .connected_server
                    .registries()?
                    .entity_types
                    .get_entry(entity.entity_type()?)
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
            });
        }

        Ok(())
    }

    #[GetInvSlot]
    pub(crate) fn get_inv_slot(&self, slot: usize) -> ActorResult<ItemStack> {
        self.associated_data.inventory.get_slot(slot)
    }

    #[SetInvSlot]
    pub(crate) fn set_inv_slot(&mut self, slot: usize, item: ItemStack) -> ActorResult<()> {
        let copy = item.clone();

        let slot = self
            .associated_data
            .screen
            .as_ref()
            .map(|(x, _)| x.container_slot_count())
            .unwrap_or(0)
            + slot;
        self.associated_data.inventory.set_slot(slot, copy)?;

        let slot = if self.associated_data.screen.is_some() {
            (slot as i32) - 9
        } else {
            slot as i32
        };

        if slot < 0 {
            return Err(ActorError::IndexOutOfBounds);
        }

        let window_id = self
            .associated_data
            .screen
            .as_ref()
            .map(|_| self.associated_data.window_id as i32)
            .unwrap_or(0);

        let packet = ContainerSetSlotS2CPlayPacket {
            window_id: VarInt::new(window_id),
            state_id: VarInt::new(1),
            slot: slot as i16,
            slot_data: item.into(),
        };
        self.write_packet(packet);
        Ok(())
    }

    #[Username]
    pub fn username(&self) -> ActorResult<String> {
        Ok(self.associated_data.username.clone())
    }

    #[Uuid]
    pub fn uuid(&self) -> ActorResult<Uuid> {
        Ok(self.associated_data.uuid)
    }

    #[Position]
    pub fn position(&self) -> ActorResult<Vec3<f64>> {
        Ok(self.associated_data.last_position)
    }

    #[SendMessage]
    pub(crate) fn send_message_component(&mut self, message: TextComponent) -> ActorResult<()> {
        self.write_packet(SystemChatS2CPlayPacket {
            content: PtcText::from(message).to_nbt(),
            is_actionbar: false,
        });
        Ok(())
    }

    #[SendActionBar]
    pub(crate) fn send_action_bar_component(&mut self, message: TextComponent) -> ActorResult<()> {
        self.write_packet(SystemChatS2CPlayPacket {
            content: PtcText::from(message).to_nbt(),
            is_actionbar: true,
        });
        Ok(())
    }

    #[OpenScreen]
    pub fn open_screen(&mut self, kind: ScreenWindowKind) -> ActorResult<()> {
        let id = if self.associated_data.window_id > 100 {
            self.associated_data.window_id = 1;
            1
        } else {
            self.associated_data.window_id += 1;
            self.associated_data.window_id
        };
        self.write_packet(OpenScreenS2CPlayPacket {
            window: VarInt::new(id as i32),
            title: PtcText::new().to_nbt(),
            kind,
        });
        self.associated_data.screen = Some((
            kind,
            DataInventory::new_filled(kind.container_slot_count(), ItemStack::air),
        ));
        Ok(())
    }

    #[SetScreenSlot]
    pub fn set_screen_slot(&mut self, slot: usize, item: ItemStack) -> ActorResult<()> {
        let Some(inventory) = self.associated_data.screen.as_mut().map(|x| &mut x.1) else {
            return Err(ActorError::BadRequest);
        };
        inventory.set_slot(slot, item.clone())?;

        self.write_packet(ContainerSetSlotS2CPlayPacket {
            window_id: VarInt::new(self.associated_data.window_id as i32),
            state_id: VarInt::new(0),
            slot: slot as i16,
            slot_data: item.into(),
        });

        Ok(())
    }

    #[GetScreenSlot]
    pub fn get_screen_slot(&mut self, slot: usize) -> ActorResult<ItemStack> {
        let Some(inventory) = self.associated_data.screen.as_mut().map(|x| &mut x.1) else {
            return Err(ActorError::BadRequest);
        };
        inventory.get_slot(slot)
    }

    #[PlaySound]
    pub fn play_sound(&mut self, sound: Sound) -> ActorResult<()> {
        self.write_packet(SoundEntityS2CPlayPacket {
            sound: sound.clone().into(),
            category: SoundCategory::Master,
            entity: self.associated_data.entity_id.into(),
            volume: sound.volume,
            pitch: sound.pitch,
            seed: 0,
        });
        Ok(())
    }
}

impl Player {
    pub(crate) fn write_packet<P: PrefixedPacketEncode + std::fmt::Debug>(
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

        self.send_packet_buf(buf)?;

        Ok(())
    }

    pub fn inventory(&self) -> ActorResult<PlayerInventory> {
        Ok(PlayerInventory {
            player: self.clone(),
        })
    }

    pub fn send_message(&self, text: impl Text) -> ActorResult<()> {
        self.send_message_component(text.into())
    }

    pub fn send_action_bar(&self, text: impl Text) -> ActorResult<()> {
        self.send_action_bar_component(text.into())
    }
}

impl ConnectionData {
    pub fn write_packet<P: PrefixedPacketEncode + std::fmt::Debug>(&mut self, packet: P) {
        let mut buf = PacketBuf::new();
        packet.encode_prefixed(&mut buf).unwrap();

        let mut len_buf = PacketBuf::new();
        VarInt::from(buf.iter().count())
            .encode(&mut len_buf)
            .unwrap();

        let mut new_buf = PacketBuf::new();
        new_buf.write_u8s(len_buf.as_slice());
        new_buf.write_u8s(buf.as_slice());

        let _ = self.send_packet_buf(buf);
    }

    pub fn update_self_entity(&mut self) {
        let dim = self.associated_data.dimension.clone().unwrap();
        let pos = self.associated_data.last_position;
        let dir = self.associated_data.last_direction;
        let uuid = self.associated_data.uuid;

        Runtime::spawn_task(move || {
            dim.get_entity(uuid).teleport(pos)?;
            dim.get_entity(uuid).rotate(dir)
        });
    }
}

#[derive(Debug)]
pub struct ConnectionWithSignal {
    pub(crate) player: Player,
    pub(crate) _signal: Receiver<ConnectionStoppedSignal>,
    pub(crate) stage: Arc<Mutex<Stage>>,
    pub(crate) is_loaded: Arc<AtomicBool>,
}

impl ConnectionWithSignal {
    pub fn lower(&self) -> Player {
        self.player.clone()
    }
}
