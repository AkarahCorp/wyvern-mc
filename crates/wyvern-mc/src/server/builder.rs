use super::Server;

pub struct ServerBuilder {

}

impl ServerBuilder {
    pub fn new() -> ServerBuilder {
        ServerBuilder {  }
    }

    pub fn start(self) {
        let server = Server {
            connections: Vec::new()
        };
        server.start();
    }
}