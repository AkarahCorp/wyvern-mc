use tokio::signal;

use crate::server::ServerBuilder;

pub struct ProxyBuilder {
    servers: Vec<ServerBuilder>,
}

impl Default for ProxyBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl ProxyBuilder {
    pub fn new() -> ProxyBuilder {
        ProxyBuilder {
            servers: Vec::new(),
        }
    }

    pub fn with_server(&mut self, server: ServerBuilder) {
        self.servers.push(server);
    }

    pub async fn start_all(self) {
        for server in self.servers {
            tokio::spawn(server.start());
        }

        loop {
            match signal::ctrl_c().await {
                Ok(()) => {
                    tokio::task::yield_now().await;
                    break;
                }
                Err(err) => {
                    eprintln!("Unable to listen for shutdown signal: {}", err);
                    break;
                }
            }
        }
    }
}
