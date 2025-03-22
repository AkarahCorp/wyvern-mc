use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};

use crate::{
    blocks::BlockState,
    entities::{Entity, EntityComponents, EntityData, PlayerSkinData},
    server::registries::RegistryKeys,
};
use chunk::{Chunk, ChunkSection};
use flume::Sender;
use voxidian_protocol::{
    packet::s2c::play::{
        AddEntityS2CPlayPacket, BlockUpdateS2CPlayPacket, PlayerActionEntry,
        PlayerInfoUpdateS2CPlayPacket, RemoveEntitiesS2CPlayPacket,
    },
    registry::RegEntry,
    value::{
        Angle, BlockPos, EntityType as PtcEntityType, Identifier, ProfileProperty, Uuid, VarInt,
    },
};
use wyvern_actors::Actor;
use wyvern_components::{ComponentElement, DataComponentHolder, DataComponentMap};
use wyvern_values::id;

use crate::{events::ChunkLoadEvent, runtime::Runtime, server::Server};

use crate::actors::{ActorError, ActorResult};
use wyvern_values::{Id, Vec2, Vec3};

pub mod chunk;
mod update;

#[allow(dead_code)]
#[crate::actor(Dimension, DimensionMessage)]
pub struct DimensionData {
    #[allow(unused)]
    pub(crate) name: Id,
    pub(crate) chunks: HashMap<Vec2<i32>, Chunk>,
    pub(crate) entities: HashMap<Uuid, EntityData>,
    pub(crate) server: Option<Server>,
    pub(crate) sender: Sender<DimensionMessage>,
    pub(crate) dim_type: Id,
    pub(crate) chunk_generator: Box<dyn Fn(&mut Chunk, i32, i32) + Send>,
    pub(crate) chunk_max: (u32, u32),
    pub(crate) last_update: Instant,
}

impl Dimension {
    pub fn get_entity(&self, entity: Uuid) -> Entity {
        Entity {
            uuid: entity,
            dimension: self.clone(),
        }
    }
}

impl DimensionData {
    pub fn event_loop(mut self) {
        loop {
            self.handle_messages();
            if Instant::now().duration_since(self.last_update) > Duration::from_millis(50) {
                self.last_update = Instant::now();
                let _ = self.auto_apply_entity_properties();
                let _ = self.propogate_entity_packets();
            }
        }
    }
}

#[crate::message(Dimension, DimensionMessage)]
impl DimensionData {
    #[GetEntityById]
    pub fn get_entity_by_id(&self, id: i32) -> ActorResult<Entity> {
        self.entities
            .iter()
            .find(|x| x.1.get(EntityComponents::ENTITY_ID) == Ok(id))
            .map(|x| Entity {
                dimension: self.as_actor(),
                uuid: *x.0,
            })
            .ok_or(ActorError::IndexOutOfBounds)
    }

    #[GetName]
    #[doc = "Get the name of this dimension."]
    pub fn name(&self) -> ActorResult<Id> {
        Ok(self.name.clone())
    }

    #[GetServer]
    #[doc = "Get the server this Dimension is running under."]
    pub fn server(&self) -> ActorResult<Server> {
        self.server.clone().ok_or(ActorError::ActorIsNotLoaded)
    }

    #[GetChunkSection]
    #[doc = "Returns a copy of the 16x16x16 chunk section at the provided coordinates."]
    pub fn get_chunk_section(&mut self, position: Vec3<i32>) -> ActorResult<Option<ChunkSection>> {
        let chunk_pos = Vec2::new(position.x(), position.z());
        self.try_initialize_chunk(&chunk_pos)?;

        match self.chunks.get_mut(&chunk_pos) {
            Some(chunk) => {
                let chunk_y = position.y() / 16;
                Ok(Some(chunk.section_at_mut(chunk_y).unwrap().clone()))
            }
            None => Ok(None),
        }
    }

    #[SetBlock]
    #[doc = "Sets a block in this dimension at the given coordinates to the provided block state."]
    pub fn set_block(&mut self, position: Vec3<i32>, block_state: BlockState) -> ActorResult<()> {
        let chunk_pos = Vec2::new(position.x().div_euclid(16), position.z().div_euclid(16));
        let pos_in_chunk = Vec3::new(
            position.x().rem_euclid(16),
            position.y(),
            position.z().rem_euclid(16),
        );

        self.try_initialize_chunk(&chunk_pos)?;

        let Some(chunk) = self.chunks.get_mut(&chunk_pos) else {
            return Ok(());
        };
        chunk.set_block_at(pos_in_chunk, block_state.clone());

        let server = self.server.clone().unwrap();
        Runtime::spawn_task(async move {
            for conn in server.players().unwrap_or_else(|_| Vec::new()) {
                let block_state = block_state.clone();
                let pos = position;
                let conn = conn.clone();

                let _ = conn.write_packet(BlockUpdateS2CPlayPacket {
                    pos: BlockPos::new(pos.x(), pos.y(), pos.z()),
                    block: unsafe { RegEntry::new_unchecked(block_state.protocol_id() as u32) },
                });
            }
            Ok(())
        });
        Ok(())
    }

    #[GetBlock]
    #[doc = "Returns a copy of the block state at the provided coordinates."]
    pub fn get_block(&mut self, position: Vec3<i32>) -> ActorResult<BlockState> {
        let chunk = Vec2::new(position.x().div_euclid(16), position.z().div_euclid(16));
        let pos_in_chunk = Vec3::new(
            position.x().rem_euclid(16),
            position.y(),
            position.z().rem_euclid(16),
        );

        self.try_initialize_chunk(&chunk)?;

        let chunk = self.chunks.get_mut(&chunk).unwrap();
        Ok(chunk.get_block_at(pos_in_chunk))
    }

    #[GetDimType]
    #[doc = "Returns the Dimension Type value of this Dimension."]
    pub fn dimension_type(&mut self) -> ActorResult<Id> {
        Ok(self.dim_type.clone())
    }

    #[SetChunkGenerator]
    #[doc = "Overrides the function that will be called whenever a new Chunk is generated. The default chunk generator is a no-op."]
    pub fn set_boxed_chunk_generator(
        &mut self,
        function: Box<dyn Fn(&mut Chunk, i32, i32) + Send>,
    ) -> ActorResult<()> {
        self.chunk_generator = function;
        Ok(())
    }

    #[GetAllEntities]
    #[doc = "Returns a handle to all of the entities present in this dimension."]
    pub fn entities(&self) -> ActorResult<Vec<Entity>> {
        Ok(self
            .entities
            .values()
            .filter(|x| !x.get(EntityComponents::PLAYER_CONTROLLED).unwrap())
            .map(|x| Entity {
                dimension: self.as_actor(),
                uuid: x.get(EntityComponents::UUID).unwrap(),
            })
            .collect())
    }

    #[GetAllEntitiesAndHumans]
    #[doc = "Returns a handle to all of the entities present in this dimension, including human entities."]
    pub fn all_entities(&self) -> ActorResult<Vec<Entity>> {
        Ok(self
            .entities
            .values()
            .map(|x| Entity {
                dimension: self.as_actor(),
                uuid: x.components.get(EntityComponents::UUID).unwrap(),
            })
            .collect())
    }

    #[SpawnEntity]
    #[doc = "Spawns a new entity in the dimension with the given type, returning a handle to the entity."]
    pub fn spawn_entity(&mut self, entity_type: Id) -> ActorResult<Entity> {
        let mut uuid = Uuid::new_v4();
        while self.entities.contains_key(&uuid) {
            uuid = Uuid::new_v4();
        }

        let mut components = DataComponentMap::new();
        let id = self.server.clone().unwrap().new_entity_id()?;
        components.set(EntityComponents::ENTITY_ID, id);
        components.set(EntityComponents::UUID, uuid);
        components.set(EntityComponents::ENTITY_TYPE, entity_type.clone());
        components.set(EntityComponents::POSITION, Vec3::new(0.0, 0.0, 0.0));
        components.set(EntityComponents::DIRECTION, Vec2::new(0.0, 0.0));
        components.set(EntityComponents::VELOCITY, Vec3::new(0.0, 0.0, 0.0));
        components.set(EntityComponents::PLAYER_CONTROLLED, false);

        self.entities.insert(
            uuid,
            EntityData {
                last_components: DataComponentMap::new(),
                components,
            },
        );

        let dim = self.as_actor();

        Runtime::spawn_task(async move {
            for conn in dim.players().unwrap_or_else(|_| Vec::new()) {
                let conn = dim.server().unwrap().player(conn).unwrap();
                let _ = conn.write_packet(AddEntityS2CPlayPacket {
                    id: id.into(),
                    uuid,
                    kind: PtcEntityType::vanilla_registry()
                        .get_entry(&entity_type.clone().into())
                        .unwrap(),
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                    pitch: Angle::of_deg(0.0),
                    yaw: Angle::of_deg(0.0),
                    head_yaw: Angle::of_deg(0.0),
                    data: VarInt::from(0),
                    vel_x: 0,
                    vel_y: 0,
                    vel_z: 0,
                });
            }

            Ok(())
        });

        Ok(Entity {
            dimension: self.as_actor(),
            uuid,
        })
    }

    #[SpawnHumanEntity]
    #[doc = "Spawns a new entity in the dimension with the given type, returning a handle to the entity."]
    pub fn spawn_human_entity(&mut self, skin: PlayerSkinData) -> ActorResult<Entity> {
        let mut uuid = Uuid::new_v4();
        while self.entities.contains_key(&uuid) {
            uuid = Uuid::new_v4();
        }

        let mut components = DataComponentMap::new();
        let id = self.server.clone().unwrap().new_entity_id()?;
        components.set(EntityComponents::ENTITY_ID, id);
        components.set(EntityComponents::UUID, uuid);
        components.set(EntityComponents::ENTITY_TYPE, id![minecraft:player]);
        components.set(EntityComponents::POSITION, Vec3::new(0.0, 0.0, 0.0));
        components.set(EntityComponents::DIRECTION, Vec2::new(0.0, 0.0));
        components.set(EntityComponents::VELOCITY, Vec3::new(0.0, 0.0, 0.0));
        components.set(EntityComponents::PLAYER_CONTROLLED, false);
        components.set(EntityComponents::PLAYER_SKIN, skin.clone());

        self.entities.insert(
            uuid,
            EntityData {
                last_components: DataComponentMap::new(),
                components,
            },
        );

        let dim = self.as_actor();

        Runtime::spawn_task(async move {
            for conn in dim.players().unwrap_or_else(|_| Vec::new()) {
                let conn = dim.server().unwrap().player(conn).unwrap();
                let name = format!("NPC_{:?}", id);
                let props = vec![ProfileProperty {
                    name: "textures".to_string(),
                    value: skin.texture.clone(),
                    sig: Some(skin.signature.clone()),
                }];
                let _ = conn.write_packet(PlayerInfoUpdateS2CPlayPacket {
                    actions: vec![(
                        uuid,
                        vec![PlayerActionEntry::AddPlayer {
                            name,
                            props: props.into(),
                        }],
                    )],
                });
                let _ = conn.write_packet(AddEntityS2CPlayPacket {
                    id: id.into(),
                    uuid,
                    kind: PtcEntityType::vanilla_registry()
                        .get_entry(&Identifier::new_const("minecraft", "player"))
                        .unwrap(),
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                    pitch: Angle::of_deg(0.0),
                    yaw: Angle::of_deg(0.0),
                    head_yaw: Angle::of_deg(0.0),
                    data: VarInt::from(0),
                    vel_x: 0,
                    vel_y: 0,
                    vel_z: 0,
                });
            }

            Ok(())
        });

        Ok(Entity {
            dimension: self.as_actor(),
            uuid,
        })
    }

    #[SpawnPlayerEntity]
    pub(crate) fn spawn_player_entity(&mut self, uuid: Uuid, id: i32) -> ActorResult<Entity> {
        let mut components = DataComponentMap::new();
        components.set(EntityComponents::ENTITY_ID, id);
        components.set(EntityComponents::UUID, uuid);
        components.set(
            EntityComponents::ENTITY_TYPE,
            Id::constant("minecraft", "player"),
        );
        components.set(EntityComponents::POSITION, Vec3::new(0.0, 0.0, 0.0));
        components.set(EntityComponents::DIRECTION, Vec2::new(0.0, 0.0));
        components.set(EntityComponents::VELOCITY, Vec3::new(0.0, 0.0, 0.0));
        components.set(EntityComponents::PLAYER_CONTROLLED, true);
        self.entities.insert(
            uuid,
            EntityData {
                last_components: DataComponentMap::new(),
                components,
            },
        );

        let dim = self.as_actor();

        Runtime::spawn_task(async move {
            for conn in dim.players().unwrap_or_else(|_| Vec::new()) {
                if conn != uuid {
                    let conn = dim.server().unwrap().player(conn).unwrap();
                    let _ = conn.write_packet(AddEntityS2CPlayPacket {
                        id: id.into(),
                        uuid,
                        kind: PtcEntityType::vanilla_registry()
                            .get_entry(&Identifier::new("minecraft", "player"))
                            .unwrap(),
                        x: 0.0,
                        y: 0.0,
                        z: 0.0,
                        pitch: Angle::of_deg(0.0),
                        yaw: Angle::of_deg(0.0),
                        head_yaw: Angle::of_deg(0.0),
                        data: VarInt::from(0),
                        vel_x: 0,
                        vel_y: 0,
                        vel_z: 0,
                    });
                }
            }
            Ok(())
        });

        Ok(Entity {
            dimension: self.as_actor(),
            uuid,
        })
    }

    #[RemoveEntity]
    pub(crate) fn remove_entity(&mut self, uuid: Uuid) -> ActorResult<()> {
        let entry = self.entities.remove(&uuid);

        if let Some(entry) = entry {
            let server = self
                .server
                .as_ref()
                .ok_or(ActorError::ActorDoesNotExist)?
                .clone();

            Runtime::spawn_task(async move {
                for conn in server.players()? {
                    let _ = conn.write_packet(RemoveEntitiesS2CPlayPacket {
                        entities: vec![VarInt::new(
                            entry.get(EntityComponents::ENTITY_ID).unwrap(),
                        )]
                        .into(),
                    });
                }
                Ok(())
            });
        };

        Ok(())
    }

    #[SetEntityComponent]
    pub(crate) fn set_entity_component_unchecked(
        &mut self,
        uuid: Uuid,
        id: Id,
        value: Arc<dyn ComponentElement>,
    ) -> ActorResult<()> {
        if let Some(entity) = self.entities.get_mut(&uuid) {
            entity.components.inner_mut().insert(id, value);
        }
        Ok(())
    }

    #[GetEntityComponent]
    pub(crate) fn get_entity_component_unchecked(
        &mut self,
        uuid: Uuid,
        id: Id,
    ) -> ActorResult<Arc<dyn ComponentElement>> {
        self.entities
            .get_mut(&uuid)
            .and_then(|entity| entity.components.inner().get(&id))
            .ok_or(ActorError::ComponentNotFound)
            .cloned()
    }

    #[GetPlayers]
    #[doc = "Returns the UUID for all players present in this dimension."]
    pub fn players(&mut self) -> ActorResult<Vec<Uuid>> {
        let mut vec = Vec::new();
        for entity in &mut self.entities {
            if entity.1.get(EntityComponents::PLAYER_CONTROLLED)? {
                let uuid = entity.1.get(EntityComponents::UUID).unwrap();
                vec.push(uuid);
            }
        }
        Ok(vec)
    }

    #[SetChunkLimits]
    #[doc = "Sets the maximum number of chunks this dimension can hold."]
    pub fn max_chunks(&mut self, x: u32, y: u32) -> ActorResult<()> {
        self.chunk_max = (x, y);
        Ok(())
    }
}

impl Dimension {
    pub fn set_chunk_generator(
        &self,
        function: impl Fn(&mut Chunk, i32, i32) + Send + 'static,
    ) -> ActorResult<()> {
        self.set_boxed_chunk_generator(Box::new(function))
    }
}

impl DimensionData {
    pub(crate) fn new(name: Id, server: Server, dim_type: Id) -> DimensionData {
        let chan = flume::unbounded();
        DimensionData {
            name,
            chunks: HashMap::new(),
            entities: HashMap::new(),
            server: Some(server),
            receiver: chan.1,
            sender: chan.0,
            dim_type,
            chunk_generator: Box::new(|_, _, _| {}),
            chunk_max: (i32::MAX as u32, i32::MAX as u32),
            last_update: Instant::now(),
        }
    }

    pub(crate) fn try_initialize_chunk(&mut self, pos: &Vec2<i32>) -> ActorResult<()> {
        if !self.chunks.contains_key(pos)
            && pos.x() <= self.chunk_max.0 as i32
            && pos.y() <= self.chunk_max.1 as i32
        {
            let server = self.server.clone().unwrap();
            let registries = server.registries()?;

            let dim_type = registries
                .get(RegistryKeys::DIMENSION_TYPE)
                .get(self.dim_type.clone())
                .unwrap();

            let min_sections = dim_type.min_y / 16;
            let max_sections = (dim_type.min_y + dim_type.height as i32) / 16;

            let mut chunk = Chunk::new(min_sections, max_sections);
            (self.chunk_generator)(&mut chunk, pos.x(), pos.y());
            self.chunks.insert(*pos, chunk);

            server.spawn_event(ChunkLoadEvent {
                dimension: self.as_actor(),
                pos: *pos,
            })?;
        }
        Ok(())
    }
}
