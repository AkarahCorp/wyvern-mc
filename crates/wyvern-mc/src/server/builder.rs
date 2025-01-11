use crate::systems::{intos::IntoSystem, system::System};

use super::ServerData;

pub struct ServerBuilder {
    systems: Vec<Box<dyn System + Send + Sync + 'static>>
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
        <S as IntoSystem<I>>::System: Send + Sync + 'static {
        self.systems.push(Box::new(s.into_system()));
    }

    pub async fn start(self) {
        let server = ServerData {
            connections: Vec::new(),
            systems: self.systems
        };

        server.start().await;
    }
}