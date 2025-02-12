use crate::{runtime::Runtime, server::ServerBuilder};

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
            Runtime::spawn(server.start());
        }

        loop {
            futures_lite::future::yield_now().await;
        }
    }
}
