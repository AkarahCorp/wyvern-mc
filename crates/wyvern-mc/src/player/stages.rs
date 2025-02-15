use voxidian_protocol::{
    packet::{
        Stage,
        c2s::{
            config::C2SConfigPackets,
            login::C2SLoginPackets,
            play::{BlockFace, C2SPlayPackets, PlayerStatus},
            status::C2SStatusPackets,
        },
        s2c::{
            config::{
                FinishConfigurationS2CConfigPacket, KnownPack, SelectKnownPacksS2CConfigPacket,
            },
            login::LoginFinishedS2CLoginPacket,
            play::{
                AddEntityS2CPlayPacket, DisconnectS2CPlayPacket, GameEvent, GameEventS2CPlayPacket,
                Gamemode, LoginS2CPlayPacket, PlayerActionEntry, PlayerInfoUpdateS2CPlayPacket,
                PlayerPositionS2CPlayPacket, PongResponseS2CPlayPacket, TeleportFlags,
            },
            status::{
                PongResponseS2CStatusPacket, StatusResponse, StatusResponsePlayers,
                StatusResponseVersion,
            },
        },
    },
    registry::RegEntry,
    value::{Angle, Identifier, LengthPrefixHashMap, Text, TextComponent, VarInt},
};

use crate::{
    actors::{Actor, ActorError, ActorResult},
    dimension::{Dimension, blocks::BlockState},
    events::{
        BreakBlockEvent, ChangeHeldSlotEvent, ChatMessageEvent, DropItemEvent, PlaceBlockEvent,
        PlayerCommandEvent, PlayerJoinEvent, PlayerMoveEvent, SwapHandsEvent,
    },
    inventory::{ITEM_REGISTRY, Inventory, ItemStack},
    runtime::Runtime,
    values::{Key, Vec3, cell::Token},
};

use super::{ConnectionData, Player};

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

    pub async fn login_stage(&mut self) -> ActorResult<()> {
        self.read_packets(async |packet: C2SLoginPackets, this: &mut Self| {
            log::debug!("Packet: {:?}", packet);
            match packet {
                C2SLoginPackets::CustomQueryAnswer(_packet) => todo!(),
                C2SLoginPackets::LoginAcknowledged(_packet) => {
                    *this.stage.lock().unwrap() = Stage::Config;
                    this.write_packet(SelectKnownPacksS2CConfigPacket {
                        known_packs: vec![KnownPack {
                            namespace: "minecraft".to_string(),
                            id: "core".to_string(),
                            version: "1.21.4".to_string(),
                        }]
                        .into(),
                    })
                    .await;
                }
                C2SLoginPackets::Key(_packet) => todo!(),
                C2SLoginPackets::Hello(packet) => {
                    this.associated_data.username = packet.username.clone();
                    this.associated_data.uuid = packet.uuid;
                    this.write_packet(LoginFinishedS2CLoginPacket {
                        uuid: packet.uuid,
                        username: packet.username,
                        props: LengthPrefixHashMap::new(),
                    })
                    .await;
                }
                C2SLoginPackets::CookieResponse(_packet) => todo!(),
            }

            Ok(())
        })
        .await
    }

    pub async fn configuration_stage(&mut self) -> ActorResult<()> {
        self.read_packets(async |packet: C2SConfigPackets, this: &mut Self| {
            log::debug!("Packet: {:?}", packet);
            {
                match packet {
                    C2SConfigPackets::CustomPayload(_packet) => {}
                    C2SConfigPackets::FinishConfiguration(_packet) => {
                        *this.stage.lock().unwrap() = Stage::Play;
                        this.associated_data.entity_id =
                            this.connected_server.new_entity_id().await?;
                        this.write_packet(LoginS2CPlayPacket {
                            entity: this.associated_data.entity_id,
                            hardcore: false,
                            // fake dimensions so we can control client w/o extra storage
                            dims: vec![Identifier::new("wyvern", "fake")].into(),
                            max_players: VarInt::from(100),
                            view_dist: VarInt::from(16),
                            sim_dist: VarInt::from(16),
                            reduced_debug: false,
                            respawn_screen: true,
                            limited_crafting: false,
                            // TODO: Turn this into an actual Dimension Type lookup for
                            // the root dimension
                            dim: unsafe { RegEntry::new_unchecked(0) },
                            dim_name: Identifier::new("wyvern", "fake"),
                            seed: 0,
                            gamemode: Gamemode::Survival,
                            old_gamemode: Gamemode::None,
                            is_debug: false,
                            is_flat: false,
                            death_loc: None,
                            portal_cooldown: VarInt::from(0),
                            sea_level: VarInt::from(64),
                            enforce_chat_reports: false,
                        })
                        .await;

                        this.write_packet(PlayerPositionS2CPlayPacket {
                            teleport_id: VarInt::from(0),
                            x: 1.0,
                            y: 32.0,
                            z: 2.0,
                            vx: 0.0,
                            vy: 0.5,
                            vz: 0.0,
                            adyaw_deg: 0.0,
                            adpitch_deg: 0.0,
                            flags: TeleportFlags {
                                relative_x: false,
                                relative_y: false,
                                relative_z: false,
                                relative_pitch: false,
                                relative_yaw: false,
                                relative_vx: false,
                                relative_vy: false,
                                relative_vz: false,
                                rotate_velocity: false,
                            },
                        })
                        .await;
                    }
                    C2SConfigPackets::ResourcePack(_packet) => todo!(),
                    C2SConfigPackets::CookieResponse(_packet) => todo!(),
                    C2SConfigPackets::Pong(_packet) => todo!(),
                    C2SConfigPackets::ClientInformation(packet) => {
                        this.associated_data.render_distance = packet.info.view_distance as i32;
                    }
                    C2SConfigPackets::KeepAlive(_packet) => todo!(),
                    C2SConfigPackets::SelectKnownPacks(_packet) => {
                        this.write_packet(
                            this.connected_server
                                .registries()
                                .await?
                                .biomes
                                .inner
                                .to_registry_data_packet(),
                        )
                        .await;
                        this.write_packet(
                            this.connected_server
                                .registries()
                                .await?
                                .damage_types
                                .inner
                                .to_registry_data_packet(),
                        )
                        .await;
                        this.write_packet(
                            this.connected_server
                                .registries()
                                .await?
                                .wolf_variants
                                .inner
                                .to_registry_data_packet(),
                        )
                        .await;

                        this.write_packet(
                            this.connected_server
                                .registries()
                                .await?
                                .painting_variants
                                .inner
                                .to_registry_data_packet(),
                        )
                        .await;

                        this.write_packet(
                            this.connected_server
                                .registries()
                                .await?
                                .dimension_types
                                .inner
                                .to_registry_data_packet(),
                        )
                        .await;

                        this.write_packet(FinishConfigurationS2CConfigPacket).await;
                    }
                }
            }

            Ok(())
        })
        .await
    }

    pub async fn play_phase(&mut self) -> ActorResult<()> {
        self.read_packets(
            async |packet: C2SPlayPackets, this: &mut Self| -> ActorResult<()> {
                log::debug!(
                    "Player {:?} has sent packet: {:?}",
                    this.associated_data.username,
                    packet
                );

                match packet {
                    C2SPlayPackets::PlayerLoaded(_packet) => {
                        this.associated_data.is_loaded = true;
                    }
                    C2SPlayPackets::ChatCommand(packet) => {
                        this.connected_server.spawn_event(PlayerCommandEvent {
                            player: Player {
                                sender: this.sender.clone(),
                            },
                            command: packet.command,
                        })?;
                    }
                    C2SPlayPackets::PlayerAction(packet) => {
                        log::warn!("{:?}", packet);
                        match packet.status {
                            PlayerStatus::StartedDigging => {}
                            PlayerStatus::CancelledDigging => {}
                            PlayerStatus::FinishedDigging => {
                                let block = Vec3::new(
                                    packet.location.x,
                                    packet.location.y,
                                    packet.location.z,
                                );
                                this.associated_data
                                    .dimension
                                    .as_ref()
                                    .unwrap()
                                    .set_block(
                                        block,
                                        BlockState::new(Key::constant("minecraft", "air")),
                                    )
                                    .await?;
                                this.connected_server.spawn_event(BreakBlockEvent {
                                    player: Player {
                                        sender: this.sender.clone(),
                                    },
                                    position: block,
                                })?;
                            }
                            PlayerStatus::DropItemStack => {
                                let item = this
                                    .get_inv_slot(this.associated_data.held_slot as usize)
                                    .await?;
                                this.set_inv_slot(
                                    this.associated_data.held_slot as usize,
                                    ItemStack::air(),
                                )
                                .await?;
                                this.connected_server.spawn_event(DropItemEvent {
                                    player: Player {
                                        sender: this.sender.clone(),
                                    },
                                    item,
                                })?;
                            }
                            PlayerStatus::DropItem => {
                                let item = this
                                    .get_inv_slot(this.associated_data.held_slot as usize)
                                    .await?;
                                this.set_inv_slot(
                                    this.associated_data.held_slot as usize,
                                    ItemStack::air(),
                                )
                                .await?;
                                this.connected_server.spawn_event(DropItemEvent {
                                    player: Player {
                                        sender: this.sender.clone(),
                                    },
                                    item,
                                })?;
                            }
                            PlayerStatus::FinishUsingItem => {}
                            PlayerStatus::SwapItems => {
                                this.connected_server.spawn_event(SwapHandsEvent {
                                    player: Player {
                                        sender: this.sender.clone(),
                                    },
                                })?;
                            }
                        }
                    }
                    C2SPlayPackets::AcceptTeleportation(packet) => {
                        if packet.teleport_id.as_i32() == 0 {
                            log::debug!("Setting dimension...");

                            let key = Key::<Dimension>::constant("null", "null");
                            let token = Token::new(Key::<Dimension>::constant("null", "null"));
                            let token_copy = token.clone();
                            this.connected_server.spawn_event(PlayerJoinEvent {
                                player: Player {
                                    sender: this.sender.clone(),
                                },
                                new_dimension: token_copy,
                            })?;

                            loop {
                                Runtime::yield_now().await;
                                this.handle_messages().await;

                                if token.get() != key {
                                    break;
                                }
                            }

                            this.associated_data.dimension =
                                this.connected_server.dimension(token.get()).await.ok();

                            if this.associated_data.dimension.is_none() {
                                let mut text = Text::new();
                                text.push(TextComponent::of_literal(
                                    "Failed to set dimension in PlayerJoinEvent",
                                ));
                                this.write_packet(DisconnectS2CPlayPacket {
                                    reason: text.to_nbt(),
                                })
                                .await;
                                return Err(ActorError::ActorIsNotLoaded);
                            }

                            log::debug!("Sending game events chunk packet...");
                            this.write_packet(GameEventS2CPlayPacket {
                                event: GameEvent::WaitForChunks,
                                value: 0.0,
                            })
                            .await;

                            log::debug!("Broadcasting this player info...");
                            for player in this.connected_server.connections().await {
                                let data = this.associated_data.clone();
                                this.intertwine(async move || {
                                    let _ = player
                                        .write_packet(PlayerInfoUpdateS2CPlayPacket {
                                            actions: vec![(data.uuid, vec![
                                                PlayerActionEntry::AddPlayer {
                                                    name: data.username.clone(),
                                                    properties: vec![].into(),
                                                },
                                                PlayerActionEntry::Listed(true),
                                            ])],
                                        })
                                        .await;
                                })
                                .await;
                            }

                            log::debug!("All done!");
                            log::debug!("Sending over current player info...");
                            for player in this.connected_server.connections().await {
                                if player.sender.same_channel(&this.sender) {
                                    this.write_packet(PlayerInfoUpdateS2CPlayPacket {
                                        actions: vec![(this.associated_data.uuid, vec![
                                            PlayerActionEntry::AddPlayer {
                                                name: this.associated_data.username.clone(),
                                                properties: vec![].into(),
                                            },
                                        ])],
                                    })
                                    .await;
                                } else {
                                    this.write_packet(PlayerInfoUpdateS2CPlayPacket {
                                        actions: vec![(player.uuid().await?, vec![
                                            PlayerActionEntry::AddPlayer {
                                                name: player.username().await?,
                                                properties: vec![].into(),
                                            },
                                        ])],
                                    })
                                    .await;
                                }
                            }

                            log::debug!("Sending all entities...");
                            for entity in this
                                .associated_data
                                .dimension
                                .as_ref()
                                .unwrap()
                                .all_entities()
                                .await?
                            {
                                let position = entity.position().await?;

                                log::debug!("Sending entity @ {:?}...", position);
                                this.write_packet(AddEntityS2CPlayPacket {
                                    id: entity.entity_id().await?.into(),
                                    uuid: *entity.uuid(),
                                    kind: this
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

                            log::debug!("Spawning human...");
                            let dim = this.associated_data.dimension.as_ref().unwrap().clone();
                            let data = this.associated_data.clone();
                            this.intertwine(async move || {
                                let _ = dim.spawn_player_entity(data.uuid, data.entity_id).await;
                            })
                            .await;
                            log::debug!("All done!");
                        }

                        this.send_chunks().await?;
                    }
                    C2SPlayPackets::MovePlayerPos(packet) => {
                        this.associated_data.last_position = this
                            .associated_data
                            .last_position
                            .with_x(packet.x)
                            .with_y(packet.y)
                            .with_z(packet.z);

                        this.send_chunks().await?;

                        this.connected_server.spawn_event(PlayerMoveEvent {
                            player: Player {
                                sender: this.sender.clone(),
                            },
                            new_position: this.associated_data.last_position,
                            new_direction: this.associated_data.last_direction,
                        })?;

                        this.associated_data
                            .dimension
                            .as_ref()
                            .unwrap()
                            .get_entity(this.associated_data.uuid)
                            .teleport(this.associated_data.last_position)
                            .await?;
                    }
                    C2SPlayPackets::MovePlayerPosRot(packet) => {
                        this.associated_data.last_position = this
                            .associated_data
                            .last_position
                            .with_x(packet.x)
                            .with_y(packet.y)
                            .with_z(packet.z);

                        this.associated_data.last_direction = this
                            .associated_data
                            .last_direction
                            .with_x(packet.pitch)
                            .with_y(packet.yaw);

                        this.associated_data
                            .dimension
                            .as_ref()
                            .unwrap()
                            .get_entity(this.associated_data.uuid)
                            .teleport(this.associated_data.last_position)
                            .await?;

                        this.associated_data
                            .dimension
                            .as_ref()
                            .unwrap()
                            .get_entity(this.associated_data.uuid)
                            .rotate(this.associated_data.last_direction)
                            .await?;

                        this.send_chunks().await?;

                        this.connected_server.spawn_event(PlayerMoveEvent {
                            player: Player {
                                sender: this.sender.clone(),
                            },
                            new_position: this.associated_data.last_position,
                            new_direction: this.associated_data.last_direction,
                        })?;
                    }
                    C2SPlayPackets::MovePlayerRot(packet) => {
                        this.associated_data.last_direction = this
                            .associated_data
                            .last_direction
                            .with_x(packet.pitch)
                            .with_y(packet.yaw);

                        this.connected_server.spawn_event(PlayerMoveEvent {
                            player: Player {
                                sender: this.sender.clone(),
                            },
                            new_position: this.associated_data.last_position,
                            new_direction: this.associated_data.last_direction,
                        })?;

                        this.associated_data
                            .dimension
                            .as_ref()
                            .unwrap()
                            .get_entity(this.associated_data.uuid)
                            .rotate(this.associated_data.last_direction)
                            .await?;
                    }
                    C2SPlayPackets::ClientInformation(packet) => {
                        this.associated_data.render_distance = packet.info.view_distance as i32;
                    }
                    C2SPlayPackets::PlayerInput(packet) => {
                        this.associated_data.input_flags = packet.flags;
                    }
                    C2SPlayPackets::ClientTickEnd(_) => {}
                    C2SPlayPackets::PingRequest(packet) => {
                        this.write_packet(PongResponseS2CPlayPacket(packet.id as u64))
                            .await;
                    }
                    C2SPlayPackets::ChunkBatchReceived(_packet) => {}
                    C2SPlayPackets::SetCreativeModeSlot(packet) => {
                        let item = ITEM_REGISTRY.lookup(&packet.new_item.id).unwrap();
                        let stack = ItemStack::new(item.id.clone().into());

                        this.associated_data
                            .inventory
                            .set_slot(packet.slot as usize, stack)
                            .await?;
                    }
                    C2SPlayPackets::SetCarriedItem(packet) => {
                        this.associated_data.held_slot = packet.slot + 36;

                        this.connected_server.spawn_event(ChangeHeldSlotEvent {
                            player: Player {
                                sender: this.sender.clone(),
                            },
                            slot: packet.slot + 36,
                        })?;
                    }
                    C2SPlayPackets::UseItemOn(packet) => {
                        let face = match packet.face {
                            BlockFace::Down => Vec3::new(0, -1, 0),
                            BlockFace::Up => Vec3::new(0, 1, 0),
                            BlockFace::North => Vec3::new(0, 0, -1),
                            BlockFace::South => Vec3::new(0, 0, 1),
                            BlockFace::West => Vec3::new(-1, 0, 0),
                            BlockFace::East => Vec3::new(1, 0, 0),
                        };
                        let target = Vec3::new(packet.target.x, packet.target.y, packet.target.z);
                        let final_pos = Vec3::new(
                            target.x() + face.x(),
                            target.y() + face.y(),
                            target.z() + face.z(),
                        );
                        let held = this
                            .associated_data
                            .inventory
                            .get_slot(this.associated_data.held_slot as usize)
                            .await?;
                        let state = BlockState::new(held.kind().retype());
                        this.associated_data
                            .dimension
                            .as_ref()
                            .unwrap()
                            .set_block(final_pos, state.clone())
                            .await?;

                        this.connected_server.spawn_event(PlaceBlockEvent {
                            player: Player {
                                sender: this.sender.clone(),
                            },
                            position: final_pos,
                            block: state,
                        })?;
                    }
                    C2SPlayPackets::Chat(packet) => {
                        this.connected_server.spawn_event(ChatMessageEvent {
                            player: Player {
                                sender: this.sender.clone(),
                            },
                            message: packet.message,
                        })?;
                    }
                    packet => {
                        log::warn!(
                            "Received unknown play packet, this packet will be ignored. {:?}",
                            packet
                        );
                    }
                };

                Ok(())
            },
        )
        .await
    }
}
