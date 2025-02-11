#![feature(try_blocks)]

pub mod components;
pub mod dimension;
pub mod events;
pub mod player;
pub mod proxy;
pub mod server;
pub mod values;

pub mod actors {
    pub trait Actor {
        fn handle_messages(&mut self) -> impl Future<Output = ()> + Send + Sync;
    }
}

pub use wyvern_macros::*;
