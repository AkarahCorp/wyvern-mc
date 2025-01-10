use crate::systems::{intos::IntoSystem, scheduler::StoredSystem};

use super::Server;

pub struct ServerBuilder {
    systems: Vec<StoredSystem>
}

impl ServerBuilder {
    pub fn new() -> ServerBuilder {
        ServerBuilder { 
            systems: Vec::new()
        }
    }

    pub fn add_system<I, S>(&mut self, s: S)
    where 
        S: IntoSystem<I>,
        <S as IntoSystem<I>>::System: 'static {
        self.systems.push(Box::new(s.into_system()));
    }

    pub async fn start(self) {
        let server = Server {
            connections: Vec::new()
        };

        server.start().await;
    }
}