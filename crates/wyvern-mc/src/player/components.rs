use std::sync::Arc;

use dyn_clone::clone_box;
use voxidian_protocol::packet::{c2s::play::InputFlags, s2c::play::Gamemode};

use crate::{
    actors::{ActorError, ActorResult},
    components::{ComponentElement, DataComponentType},
    entities::AttributeContainer,
    id,
};

use super::Player;

pub struct PlayerComponents;

impl PlayerComponents {
    pub const ATTRIBUTES: DataComponentType<AttributeContainer> =
        DataComponentType::new(id![minecraft:attributes]);
    pub const INPUT_FLAGS: DataComponentType<InputFlags> =
        DataComponentType::new(id![minecraft:input_flags]);
    pub const GAMEMODE: DataComponentType<Gamemode> =
        DataComponentType::new(id![minecraft:gamemode]);
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
