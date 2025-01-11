use tokio::sync::mpsc::Sender;

use super::message::ServerMessage;

pub struct Server {
    #[allow(dead_code)]
    pub(crate) sender: Sender<ServerMessage>
}

impl Server {
    
}