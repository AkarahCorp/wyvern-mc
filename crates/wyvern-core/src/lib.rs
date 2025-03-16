#![feature(try_blocks)]
#![allow(clippy::type_complexity)]

pub(crate) mod actors {
    pub use wyvern_actors::*;
}
pub mod blocks;
pub mod dimension;
pub mod entities;
pub mod events;
pub mod inventory;
pub mod item;
pub mod player;
pub mod plugin;
pub mod runtime;
pub mod server;

pub(crate) use wyvern_macros::*;
