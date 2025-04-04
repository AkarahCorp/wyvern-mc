use voxidian_protocol::{
    packet::{
        c2s::play::{BlockFace, C2SPlayPackets, CommandAction, InteractAction, PlayerStatus},
        s2c::play::{
            AddEntityS2CPlayPacket, AnimateS2CPlayPacket, BlockChangedAckS2CPlayPacket,
            ContainerSlotGroup, DisconnectS2CPlayPacket, EntityAnimation, GameEvent,
            GameEventS2CPlayPacket, Hand, PlayerActionEntry, PlayerInfoUpdateS2CPlayPacket,
            PongResponseS2CPlayPacket, RespawnDataKept, RespawnS2CPlayPacket, ScreenWindowKind,
            SetEntityDataS2CPlayPacket,
        },
    },
    registry::RegEntry,
    value::{Angle, ProfileProperty, Text as PtcText, TextComponent, VarInt},
};
use wyvern_components::DataComponentHolder;
use wyvern_datatypes::{gamemode::Gamemode, text::Text};

use crate::{
    actors::{Actor, ActorError, ActorResult},
    blocks::BlockState,
    entities::EntityComponents,
    events::{
        BreakBlockEvent, ChangeHeldSlotEvent, ChatMessageEvent, DropItemEvent, PlaceBlockEvent,
        PlayerAttackEntityEvent, PlayerAttackPlayerEvent, PlayerCommandEvent, PlayerJoinEvent,
        PlayerLeftClickEvent, PlayerLoadEvent, PlayerMoveEvent, PlayerRespawnEvent,
        RightClickEvent, StartBreakBlockEvent, SwapHandsEvent,
    },
    inventory::Inventory,
    item::{ITEM_REGISTRY, ItemComponents, ItemStack},
    player::{ConnectionData, HealthComponent, PlayerComponents},
    runtime::Runtime,
    server::{Server, registries::RegistryKeys},
};

use wyvern_values::{DVec3, IVec3, Id, Vec2, cell::Token, id};

impl ConnectionData {
    pub fn play_phase(&mut self) -> ActorResult<()> {
        self.read_packets(
            |packet: C2SPlayPackets, this: &mut Self| -> ActorResult<()> {
                log::debug!(
                    "Player {:?} has sent packet: {:?}",
                    this.get(PlayerComponents::USERNAME)?,
                    packet
                );

                match packet {
                    C2SPlayPackets::ChatCommand(packet) => {
                        this.connected_server.spawn_event(PlayerCommandEvent {
                            player: this.as_actor(),
                            command: packet.command,
                        })?;
                    }
                    C2SPlayPackets::PlayerAction(packet) => {
                        let block =
                            IVec3::new(packet.location.x, packet.location.y, packet.location.z);

                        this.write_packet(BlockChangedAckS2CPlayPacket(packet.sequence));
                        match packet.status {
                            PlayerStatus::StartedDigging => {
                                this.connected_server.spawn_event(StartBreakBlockEvent {
                                    player: this.as_actor(),
                                    position: block,
                                })?;
                                if this.get(PlayerComponents::GAMEMODE) == Ok(Gamemode::Creative) {
                                    let old_block = this
                                        .associated_data
                                        .dimension
                                        .as_ref()
                                        .unwrap()
                                        .get_block(block)?;

                                    this.associated_data.dimension.as_ref().unwrap().set_block(
                                        block,
                                        BlockState::new(Id::constant("minecraft", "air")),
                                    )?;
                                    this.connected_server.spawn_event(BreakBlockEvent {
                                        player: this.as_actor(),
                                        position: block,
                                        old_block,
                                    })?;
                                }
                            }
                            PlayerStatus::CancelledDigging => {}
                            PlayerStatus::FinishedDigging => {
                                if this.get(PlayerComponents::GAMEMODE) != Ok(Gamemode::Creative) {
                                    let old_block = this
                                        .associated_data
                                        .dimension
                                        .as_ref()
                                        .unwrap()
                                        .get_block(block)?;

                                    this.associated_data.dimension.as_ref().unwrap().set_block(
                                        block,
                                        BlockState::new(Id::constant("minecraft", "air")),
                                    )?;
                                    this.connected_server.spawn_event(BreakBlockEvent {
                                        player: this.as_actor(),
                                        position: block,
                                        old_block,
                                    })?;
                                }
                            }
                            PlayerStatus::DropItemStack => {
                                let item =
                                    this.get_inv_slot(this.associated_data.held_slot as usize)?;
                                this.set_inv_slot(
                                    this.associated_data.held_slot as usize,
                                    ItemStack::air(),
                                )?;
                                this.connected_server.spawn_event(DropItemEvent {
                                    player: this.as_actor(),
                                    item,
                                })?;
                            }
                            PlayerStatus::DropItem => {
                                let item =
                                    this.get_inv_slot(this.associated_data.held_slot as usize)?;
                                this.set_inv_slot(
                                    this.associated_data.held_slot as usize,
                                    ItemStack::air(),
                                )?;
                                this.connected_server.spawn_event(DropItemEvent {
                                    player: this.as_actor(),
                                    item,
                                })?;
                            }
                            PlayerStatus::FinishUsingItem => {}
                            PlayerStatus::SwapItems => {
                                this.connected_server.spawn_event(SwapHandsEvent {
                                    player: this.as_actor(),
                                })?;
                            }
                        }
                    }
                    C2SPlayPackets::AcceptTeleportation(packet) => {
                        if packet.teleport_id.as_i32() == 0 {
                            this.connect_to_new_dimension()?;
                        } else if packet.teleport_id.as_i32() != -1 {
                            this.set(
                                PlayerComponents::TELEPORT_SYNC_RECEIVED,
                                packet.teleport_id.as_i32(),
                            );
                        }

                        this.send_chunks()?;
                    }
                    C2SPlayPackets::MovePlayerPos(packet) => {
                        if this.get(PlayerComponents::TELEPORT_SYNC_SENT).unwrap_or(0)
                            > this
                                .get(PlayerComponents::TELEPORT_SYNC_RECEIVED)
                                .unwrap_or(1)
                        {
                            return Ok(());
                        }
                        this.set(
                            PlayerComponents::POSITION,
                            DVec3::new(packet.x, packet.y, packet.z),
                        );

                        this.send_chunks()?;

                        this.connected_server.spawn_event(PlayerMoveEvent {
                            player: this.as_actor(),
                            new_position: this.get(PlayerComponents::POSITION)?,
                            new_direction: this.get(PlayerComponents::DIRECTION)?,
                        })?;

                        this.update_self_entity()?;
                    }
                    C2SPlayPackets::MovePlayerPosRot(packet) => {
                        if this.get(PlayerComponents::TELEPORT_SYNC_SENT).unwrap_or(0)
                            > this
                                .get(PlayerComponents::TELEPORT_SYNC_RECEIVED)
                                .unwrap_or(1)
                        {
                            return Ok(());
                        }
                        this.set(
                            PlayerComponents::POSITION,
                            DVec3::new(packet.x, packet.y, packet.z),
                        );
                        this.set(
                            PlayerComponents::DIRECTION,
                            Vec2::new(packet.pitch, packet.yaw),
                        );

                        this.connected_server.spawn_event(PlayerMoveEvent {
                            player: this.as_actor(),
                            new_position: this.get(PlayerComponents::POSITION)?,
                            new_direction: this.get(PlayerComponents::DIRECTION)?,
                        })?;

                        this.update_self_entity()?;
                        this.send_chunks()?;
                    }
                    C2SPlayPackets::MovePlayerRot(packet) => {
                        if this.get(PlayerComponents::TELEPORT_SYNC_SENT).unwrap_or(0)
                            > this
                                .get(PlayerComponents::TELEPORT_SYNC_RECEIVED)
                                .unwrap_or(10)
                        {
                            return Ok(());
                        }
                        this.set(
                            PlayerComponents::DIRECTION,
                            Vec2::new(packet.pitch, packet.yaw),
                        );

                        this.connected_server.spawn_event(PlayerMoveEvent {
                            player: this.as_actor(),
                            new_position: this.get(PlayerComponents::POSITION)?,
                            new_direction: this.get(PlayerComponents::DIRECTION)?,
                        })?;

                        this.update_self_entity()?;
                    }
                    C2SPlayPackets::ClientInformation(packet) => {
                        this.associated_data.render_distance = packet.info.view_distance as i32;
                    }
                    C2SPlayPackets::PlayerInput(packet) => {
                        this.set(PlayerComponents::INPUT_FLAGS, packet.flags);
                    }
                    C2SPlayPackets::ClientTickEnd(_) => {}
                    C2SPlayPackets::PingRequest(packet) => {
                        this.write_packet(PongResponseS2CPlayPacket(packet.id as u64));
                    }
                    C2SPlayPackets::ChunkBatchReceived(_packet) => {}
                    C2SPlayPackets::SetCreativeModeSlot(packet) => {
                        let item_id = ITEM_REGISTRY.lookup(&packet.new_item.id).unwrap();
                        let stack = ItemStack::from(packet.new_item)
                            .with(ItemComponents::ITEM_MODEL, item_id.id.clone().into())
                            .with(
                                ItemComponents::ITEM_NAME,
                                Text::literal("Creative Mode Item"),
                            );

                        this.set_inv_slot(packet.slot as usize, stack.clone())?;
                    }
                    C2SPlayPackets::SetCarriedItem(packet) => {
                        this.associated_data.held_slot = packet.slot + 36;

                        this.connected_server.spawn_event(ChangeHeldSlotEvent {
                            player: this.as_actor(),
                            slot: packet.slot + 36,
                        })?;
                    }
                    C2SPlayPackets::UseItem(packet) => {
                        if packet.hand == Hand::Mainhand {
                            this.connected_server.spawn_event(RightClickEvent {
                                player: this.as_actor(),
                            })?;
                        }
                    }
                    C2SPlayPackets::UseItemOn(packet) => {
                        if packet.hand == Hand::Mainhand {
                            let face: IVec3 = match packet.face {
                                BlockFace::Down => IVec3::new(0, -1, 0),
                                BlockFace::Up => IVec3::new(0, 1, 0),
                                BlockFace::North => IVec3::new(0, 0, -1),
                                BlockFace::South => IVec3::new(0, 0, 1),
                                BlockFace::West => IVec3::new(-1, 0, 0),
                                BlockFace::East => IVec3::new(1, 0, 0),
                            };
                            let target =
                                IVec3::new(packet.target.x, packet.target.y, packet.target.z);
                            let final_pos = target + face;
                            let held = this
                                .associated_data
                                .inventory
                                .get_slot(this.associated_data.held_slot as usize)?;

                            let state = BlockState::new(held.kind());
                            let state_clone = state.clone();
                            let dim = this
                                .associated_data
                                .dimension
                                .as_ref()
                                .ok_or(ActorError::ActorIsNotLoaded)?
                                .clone();

                            this.write_packet(BlockChangedAckS2CPlayPacket(packet.sequence));
                            Runtime::spawn_task(async move {
                                let _ = dim.set_block(final_pos, state_clone);

                                Ok(())
                            });

                            if state.id_is_valid() {
                                if let Ok(item_count) = held.get(ItemComponents::ITEM_COUNT) {
                                    if item_count <= 1 {
                                        this.associated_data.inventory.set_slot(
                                            this.associated_data.held_slot as usize,
                                            ItemStack::air(),
                                        )?;
                                    } else {
                                        this.associated_data.inventory.set_slot(
                                            this.associated_data.held_slot as usize,
                                            held.with(ItemComponents::ITEM_COUNT, item_count - 1),
                                        )?;
                                    }
                                }
                            }

                            if state.id_is_valid() {
                                this.connected_server.spawn_event(PlaceBlockEvent {
                                    player: this.as_actor(),
                                    position: final_pos,
                                    block: state,
                                })?;
                            } else {
                                this.connected_server.spawn_event(RightClickEvent {
                                    player: this.as_actor(),
                                })?;
                            }
                        }
                    }
                    C2SPlayPackets::Chat(packet) => {
                        this.connected_server.spawn_event(ChatMessageEvent {
                            player: this.as_actor(),
                            message: packet.message,
                        })?;
                    }
                    C2SPlayPackets::ContainerClick(packet) => {
                        this.associated_data.cursor_item = packet.cursor_item.into();

                        if let Some((screen, open_inventory)) = &mut this.associated_data.screen {
                            for slot in packet.changed_slots.iter() {
                                match ScreenWindowKind::from(*screen)
                                    .get_slot_index_group(slot.slot as usize)
                                    .unwrap()
                                {
                                    ContainerSlotGroup::PlayerHotbar(hotbar) => {
                                        this.associated_data
                                            .inventory
                                            .set_slot(36 + hotbar, slot.data.clone().into())?;
                                    }
                                    ContainerSlotGroup::PlayerUpper(upper) => {
                                        this.associated_data
                                            .inventory
                                            .set_slot(9 + upper, slot.data.clone().into())?;
                                    }
                                    ContainerSlotGroup::Container(slot_idx) => {
                                        open_inventory
                                            .set_slot(slot_idx, slot.data.clone().into())?;
                                    }
                                    _ => todo!(),
                                }
                            }
                        } else {
                            for slot in packet.changed_slots.iter() {
                                this.associated_data
                                    .inventory
                                    .set_slot(slot.slot as usize, slot.data.clone().into())?;
                            }
                        }
                    }
                    C2SPlayPackets::ContainerClose(_) => {
                        this.associated_data.cursor_item = ItemStack::air();
                        this.associated_data.screen = None;
                    }
                    C2SPlayPackets::Interact(packet) => {
                        let player = this.as_actor();
                        Runtime::spawn_task(async move {
                            match packet.action {
                                InteractAction::Interact(_hand) => {}
                                InteractAction::Attack => {
                                    let victim = player
                                        .dimension()?
                                        .get_entity_by_id(packet.entity_id.into())?;
                                    if let Ok(victim) = Server::get()?.player(*victim.uuid()) {
                                        Server::get()?.spawn_event(PlayerAttackPlayerEvent {
                                            attacker: player,
                                            victim,
                                        })?;
                                    } else {
                                        Server::get()?.spawn_event(PlayerAttackEntityEvent {
                                            attacker: player,
                                            victim,
                                        })?;
                                    }
                                }
                                InteractAction::InteractAt(_, _, _, _hand) => {}
                            }
                            Ok(())
                        });
                    }
                    C2SPlayPackets::Swing(packet) => {
                        let player = this.as_actor();
                        let eid = this.associated_data.entity_id;
                        let uuid = this.get(PlayerComponents::UUID)?;
                        Server::get()?.spawn_event(PlayerLeftClickEvent {
                            player: this.as_actor(),
                        })?;
                        Runtime::spawn_task(async move {
                            let players = player.dimension()?.players()?;

                            match packet.hand {
                                Hand::Mainhand => {
                                    for player in players {
                                        if player == uuid {
                                            continue;
                                        }
                                        let player = Server::get()?.player(player)?;
                                        player.write_packet(AnimateS2CPlayPacket {
                                            id: eid.into(),
                                            anim: EntityAnimation::SwingMainHand,
                                        })?;
                                    }
                                }
                                Hand::Offhand => {
                                    for player in players {
                                        if player == uuid {
                                            continue;
                                        }
                                        let player = Server::get()?.player(player)?;
                                        player.write_packet(AnimateS2CPlayPacket {
                                            id: eid.into(),
                                            anim: EntityAnimation::SwingOffHand,
                                        })?;
                                    }
                                }
                            }
                            Ok(())
                        });
                    }
                    C2SPlayPackets::PlayerLoaded(_packet) => {
                        Server::get()?.spawn_event(PlayerLoadEvent {
                            player: this.as_actor(),
                        })?;
                    }
                    C2SPlayPackets::ClientCommand(packet) => match packet.action {
                        CommandAction::PerformRespawn => {
                            this.write_packet(RespawnS2CPlayPacket {
                                dim: unsafe { RegEntry::new_unchecked(0) },
                                dim_name: id![wyvern:fake].into(),
                                seed: 0,
                                gamemode: Gamemode::Survival.into(),
                                is_debug: false,
                                is_flat: false,
                                death_loc: None,
                                portal_cooldown: VarInt::from(0),
                                sea_level: VarInt::from(64),
                                prev_gamemode: Gamemode::Survival.into(),
                                data_kept: RespawnDataKept {
                                    keep_attributes: true,
                                    keep_metadata: true,
                                },
                            });
                            this.associated_data.loaded_chunks.clear();

                            this.set(
                                PlayerComponents::HEALTH,
                                HealthComponent {
                                    food: 20,
                                    saturation: 20.0,
                                    health: 20.0,
                                },
                            );
                            this.write_packet(GameEventS2CPlayPacket {
                                event: GameEvent::WaitForChunks,
                                value: 0.0,
                            });

                            this.set(
                                PlayerComponents::TELEPORT_POSITION,
                                this.get(PlayerComponents::POSITION)? + DVec3::new(0.0, 0.1, 0.0),
                            );
                            Server::get()?.spawn_event(PlayerRespawnEvent {
                                player: this.as_actor(),
                            })?;
                        }
                        CommandAction::RequestStats => {
                            // todo: send statistics packet
                        }
                    },
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
    }

    pub fn connect_to_new_dimension(&mut self) -> ActorResult<()> {
        log::debug!("Setting dimension...");

        let key = Id::constant("null", "null");
        let token = Token::new(Id::constant("null", "null"));
        let token_copy = token.clone();

        self.connected_server.spawn_event(PlayerJoinEvent {
            player: self.as_actor(),
            new_dimension: token_copy,
        })?;

        loop {
            self.handle_messages();

            if token.get() != key {
                break;
            }
        }

        self.associated_data.dimension = self.connected_server.dimension(token.get()).ok();

        if self.associated_data.dimension.is_none() {
            let mut text = PtcText::new();
            text.push(TextComponent::of_literal(
                "Failed to set dimension in PlayerJoinEvent",
            ));
            self.write_packet(DisconnectS2CPlayPacket {
                reason: text.to_nbt(),
            });
            return Err(ActorError::ActorIsNotLoaded);
        }

        log::debug!("Sending game events chunk packet...");
        self.write_packet(GameEventS2CPlayPacket {
            event: GameEvent::WaitForChunks,
            value: 0.0,
        });

        log::debug!("Broadcasting this player info...");
        for player in self.connected_server.connections()? {
            let uuid = self.get(PlayerComponents::UUID)?;
            let username = self.get(PlayerComponents::USERNAME)?;
            let props = if let Some(mojauth) = self.mojauth.as_ref() {
                mojauth
                    .props
                    .iter()
                    .map(|x| ProfileProperty {
                        name: x.name.clone(),
                        value: x.value.clone(),
                        sig: Some(x.sig.clone()),
                    })
                    .collect::<Vec<_>>()
            } else {
                Vec::new()
            };

            Runtime::spawn_task(async move {
                let _ = player.write_packet(PlayerInfoUpdateS2CPlayPacket {
                    actions: vec![(
                        uuid,
                        vec![
                            PlayerActionEntry::AddPlayer {
                                name: username.clone(),
                                props: props.into(),
                            },
                            PlayerActionEntry::Listed(true),
                        ],
                    )],
                });
                Ok(())
            });
        }

        log::debug!("All done!");
        log::debug!("Sending over current player info...");

        let uuid = self.get(PlayerComponents::UUID)?;
        let username = self.get(PlayerComponents::USERNAME)?;
        for player in self.connected_server.connections()? {
            if player.sender.upgrade().unwrap().same_channel(&self.sender) {
                let props = if let Some(mojauth) = self.mojauth.as_ref() {
                    mojauth
                        .props
                        .iter()
                        .map(|x| ProfileProperty {
                            name: x.name.clone(),
                            value: x.value.clone(),
                            sig: Some(x.sig.clone()),
                        })
                        .collect::<Vec<_>>()
                } else {
                    Vec::new()
                };

                self.write_packet(PlayerInfoUpdateS2CPlayPacket {
                    actions: vec![(
                        uuid,
                        vec![
                            PlayerActionEntry::AddPlayer {
                                name: username.clone(),
                                props: props.into(),
                            },
                            PlayerActionEntry::Listed(true),
                        ],
                    )],
                });
            } else {
                let uuid = player.get(PlayerComponents::UUID)?;
                let username = player.get(PlayerComponents::USERNAME)?;
                self.write_packet(PlayerInfoUpdateS2CPlayPacket {
                    actions: vec![(
                        uuid,
                        vec![
                            PlayerActionEntry::AddPlayer {
                                name: username.clone(),
                                props: player.auth_props().unwrap_or(Vec::new()).into(),
                            },
                            PlayerActionEntry::Listed(true),
                        ],
                    )],
                });
            }
        }

        let entities = self
            .associated_data
            .dimension
            .as_ref()
            .unwrap()
            .all_entities()?;
        log::debug!("Sending all entities...");
        for entity in entities {
            let position = entity
                .get(EntityComponents::POSITION)
                .unwrap_or(DVec3::new(0.0, 0.0, 0.0));
            let direction = entity
                .get(EntityComponents::DIRECTION)
                .unwrap_or(Vec2::new(0.0, 0.0));
            let id = entity.get(EntityComponents::ENTITY_ID)?;
            let ty = entity.get(EntityComponents::ENTITY_TYPE)?;

            if let Ok(skin) = entity.get(EntityComponents::PLAYER_SKIN) {
                let name = format!("NPC_{:?}", entity.get(EntityComponents::ENTITY_ID)?);
                let props = vec![ProfileProperty {
                    name: "textures".to_string(),
                    value: skin.texture,
                    sig: Some(skin.signature),
                }];
                self.write_packet(PlayerInfoUpdateS2CPlayPacket {
                    actions: vec![(
                        *entity.uuid(),
                        vec![PlayerActionEntry::AddPlayer {
                            name,
                            props: props.into(),
                        }],
                    )],
                });
            }
            self.write_packet(AddEntityS2CPlayPacket {
                id: id.into(),
                uuid: *entity.uuid(),
                kind: self
                    .connected_server
                    .registries()?
                    .get(RegistryKeys::ENTITY_TYPE)
                    .get_entry(ty)
                    .unwrap(),
                x: position[0],
                y: position[1],
                z: position[2],
                pitch: Angle::of_deg(direction[0]),
                yaw: Angle::of_deg(direction[1]),
                head_yaw: Angle::of_deg(direction[1]),
                data: VarInt::from(0),
                vel_x: 0,
                vel_y: 0,
                vel_z: 0,
            });
            self.write_packet(SetEntityDataS2CPlayPacket {
                entity: id.into(),
                data: entity.generate_metadata()?,
            });
        }

        log::debug!("Spawning human...");
        let dim = self.associated_data.dimension.as_ref().unwrap().clone();
        let uuid = self.get(PlayerComponents::UUID)?;
        let entity_id = self.associated_data.entity_id;

        Runtime::spawn_task(async move {
            let _ = dim.spawn_player_entity(uuid, entity_id);
            Ok(())
        });

        log::debug!("All done!");

        Ok(())
    }
}
