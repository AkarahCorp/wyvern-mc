use std::sync::Arc;

use dyn_clone::clone_box;
use voxidian_protocol::{packet::c2s::play::InputFlags, value::Uuid};
use wyvern_components::{ComponentElement, DataComponentType};
use wyvern_datatypes::{gamemode::Gamemode, text::Text};

use crate::{
    actors::{ActorError, ActorResult},
    entities::AttributeContainer,
};

use wyvern_values::{DVec2, DVec3, Vec2, id};

pub mod update;

use super::Player;

pub struct PlayerComponents;

impl PlayerComponents {
    pub const USERNAME: DataComponentType<String> = DataComponentType::new(id![minecraft:username]);
    pub const UUID: DataComponentType<Uuid> = DataComponentType::new(id![minecraft:uuid]);
    pub const TELEPORT_POSITION: DataComponentType<DVec3> =
        DataComponentType::new(id![minecraft:tp_position]);
    pub const TELEPORT_VELOCITY: DataComponentType<DVec3> =
        DataComponentType::new(id![minecraft:tp_velocity]);
    pub const POSITION: DataComponentType<DVec3> = DataComponentType::new(id![minecraft:position]);
    pub const DIRECTION: DataComponentType<Vec2> = DataComponentType::new(id![minecraft:direction]);
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

    pub const SIDEBAR_PRESENT: DataComponentType<bool> =
        DataComponentType::new(id![minecraft:sidebar_present]);
    pub const SIDEBAR_NAME: DataComponentType<Text> =
        DataComponentType::new(id![minecraft:sidebar_name]);
    pub const SIDEBAR_LINES: DataComponentType<Vec<Text>> =
        DataComponentType::new(id![minecraft:sidebar_lines]);

    pub const HEALTH: DataComponentType<HealthComponent> =
        DataComponentType::new(id![minecraft:health]);
    pub const WORLD_BORDER: DataComponentType<WorldBorderComponent> =
        DataComponentType::new(id![minecraft:world_border]);
    pub const EXPERIENCE: DataComponentType<ExperienceComponent> =
        DataComponentType::new(id![minecraft:experience]);
}

impl Player {
    pub fn get<T: ComponentElement>(&self, component: DataComponentType<T>) -> ActorResult<T> {
        let component = self.get_component_unchecked(component.into_name());
        let component = component?;
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

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HealthComponent {
    pub health: f32,
    pub food: i32,
    pub saturation: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WorldBorderComponent {
    pub center: DVec2,
    pub size: f64,
    pub warning_delay: i32,
    pub warning_distance: i32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ExperienceComponent {
    pub level: i32,
    pub progress: f32,
}
