use std::{pin::Pin, sync::Arc};

use crate::{
    dimension::Dimension,
    player::Player,
    server::Server,
    values::{Vec2, Vec3},
};

mod bus;
pub use bus::*;

pub trait Event {
    fn add_handler(bus: &mut EventBus, f: Box<dyn Fn(Self) -> BoxedFuture + Send + Sync>);
    fn dispatch(self, bus: Arc<EventBus>);
}

pub type BoxedFuture = Pin<Box<dyn Future<Output = ()> + Send + 'static>>;

#[derive(Debug, Clone)]
pub struct DimensionCreateEvent {
    pub dimension: Dimension,
    pub server: Server,
}

#[derive(Debug, Clone)]
pub struct ChunkLoadEvent {
    pub dimension: Dimension,
    pub pos: Vec2<i32>,
}

#[derive(Debug, Clone)]
pub struct ServerTickEvent {
    pub server: Server,
}

#[derive(Debug, Clone)]
pub struct PlayerMoveEvent {
    pub player: Player,
    pub new_position: Vec3<f64>,
    pub new_direction: Vec2<f32>,
}

#[derive(Debug, Clone)]
pub struct PlayerCommandEvent {
    pub player: Player,
    pub command: String,
}

#[derive(Debug, Clone)]
pub struct StartBreakBlockEvent {
    pub player: Player,
    pub position: Vec3<i32>,
}

#[derive(Debug, Clone)]
pub struct StopBreakBlockEvent {
    pub player: Player,
    pub position: Vec3<i32>,
}

#[derive(Debug, Clone)]
pub struct BreakBlockEvent {
    pub player: Player,
    pub position: Vec3<i32>,
}

#[derive(Debug, Clone)]
pub struct DropItemEvent {
    pub player: Player,
}

#[derive(Debug, Clone)]
pub struct DropItemStackEvent {
    pub player: Player,
}

#[derive(Debug, Clone)]
pub struct SwapHandsEvent {
    pub player: Player,
}
