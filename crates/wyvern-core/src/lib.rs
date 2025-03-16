#![feature(try_blocks)]
#![allow(clippy::type_complexity)]

pub mod actors {
    pub use wyvern_actors::*;
}
pub mod blocks;
pub mod components {
    pub use wyvern_components::*;
}
pub mod dimension;
pub mod entities;
pub mod events;
pub mod inventory;
pub mod item;
pub mod player;
pub mod runtime;
pub mod server;

pub(crate) use wyvern_macros::*;
