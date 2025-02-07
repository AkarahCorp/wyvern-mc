use futures::future::join_all;

use super::{
    BoxedFuture, ChunkLoadEvent, DimensionCreateEvent, PlayerCommandEvent, PlayerMoveEvent,
    ServerTickEvent,
};
use std::{fmt::Debug, sync::Arc};

macro_rules! event_bus {
    ($($name:ident : $t:ty)*) => {
        #[derive(Default)]
        pub struct EventBus {
            $(pub(crate) $name: Vec<Arc<Box<dyn Fn($t) -> BoxedFuture + Send + Sync>>>,)*
        }

        $(impl crate::events::Event for $t {
            fn add_handler(bus: &mut EventBus, f: Box<dyn Fn($t) -> BoxedFuture + Send + Sync>) {
                bus.$name.push(Arc::new(f));
            }

            fn dispatch(self, bus: std::sync::Arc<EventBus>) {
                tokio::spawn(async move {
                    self.dispatch_sync(bus).await;
                });
            }

            async fn dispatch_sync(self, bus: std::sync::Arc<EventBus>) {
                let futures_to_poll = bus
                        .$name
                        .clone()
                        .into_iter()
                        .map(|x| x(self.clone()));
                    join_all(futures_to_poll).await;
            }
        })*

    };
}

event_bus! {
    on_dim_create: DimensionCreateEvent
    on_server_tick: ServerTickEvent
    on_player_move: PlayerMoveEvent
    on_chunk_load: ChunkLoadEvent
    on_command: PlayerCommandEvent
}

impl Debug for EventBus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("EventBus { ... }")
    }
}
