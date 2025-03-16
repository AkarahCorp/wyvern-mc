use std::sync::Arc;

use dyn_clone::clone_box;
use voxidian_protocol::{packet::c2s::play::InputFlags, value::Uuid};
use wyvern_components::{ComponentElement, DataComponentType};
use wyvern_datatypes::gamemode::Gamemode;

use crate::{
    actors::{ActorError, ActorResult},
    entities::AttributeContainer,
};

use wyvern_values::{Vec2, Vec3, id};

use super::Player;

pub struct PlayerComponents;

impl PlayerComponents {
    pub const USERNAME: DataComponentType<String> = DataComponentType::new(id![minecraft:username]);
    pub const UUID: DataComponentType<Uuid> = DataComponentType::new(id![minecraft:uuid]);
    pub const TELEPORT_POSITION: DataComponentType<Vec3<f64>> =
        DataComponentType::new(id![minecraft:tp_position]);
    pub const POSITION: DataComponentType<Vec3<f64>> =
        DataComponentType::new(id![minecraft:position]);
    pub const DIRECTION: DataComponentType<Vec2<f32>> =
        DataComponentType::new(id![minecraft:direction]);
    pub const ATTRIBUTES: DataComponentType<AttributeContainer> =
        DataComponentType::new(id![minecraft:attributes]);
    pub const INPUT_FLAGS: DataComponentType<InputFlags> =
        DataComponentType::new(id![minecraft:input_flags]);
    pub const GAMEMODE: DataComponentType<Gamemode> =
        DataComponentType::new(id![minecraft:gamemode]);

    pub const TELEPORT_SYNC_SENT: DataComponentType<i32> =
        DataComponentType::new(id![minecraft:teleport_sent]);
    pub const TELEPORT_SYNC_RECEIVED: DataComponentType<i32> =
        DataComponentType::new(id![minecraft:teleport_sync]);
}

impl Player {
    pub fn get<T: ComponentElement>(&self, component: DataComponentType<T>) -> ActorResult<T> {
        let component = self.get_component_unchecked(component.into_name())?;

        ((*component).as_any().downcast_ref::<T>())
            .map(|x| clone_box(x))
            .map(|x| *x)
            .ok_or(ActorError::ComponentNotFound)
    }

    pub fn set<T: ComponentElement>(
        &self,
        component: DataComponentType<T>,
        value: T,
    ) -> ActorResult<()> {
        self.set_component_unchecked(component.into_name(), Arc::new(value))
    }
}
