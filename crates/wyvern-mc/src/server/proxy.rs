use tokio::sync::mpsc::Sender;

use crate::systems::typemap::TypeMap;

use super::message::ServerMessage;

#[derive(Clone)]
pub struct Server {
    #[allow(dead_code)]
    pub(crate) sender: Sender<ServerMessage>
}

impl Server {
    pub async fn fire_systems(&self, parameters: TypeMap) {
        self.sender.send(ServerMessage::FireSystems(parameters)).await.unwrap();
    }
}