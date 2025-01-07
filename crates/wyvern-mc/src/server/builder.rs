use super::Server;

pub struct ServerBuilder {

}

impl ServerBuilder {
    pub fn new() -> ServerBuilder {
        ServerBuilder {  }
    }

    pub async fn start(self) {
        let server = Server {
            connections: Vec::new()
        };
        server.start().await;
    }
}