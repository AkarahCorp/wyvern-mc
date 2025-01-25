pub trait Actor {
    fn handle_messages(&mut self) -> impl Future<Output = ()> + Send + Sync;
}
