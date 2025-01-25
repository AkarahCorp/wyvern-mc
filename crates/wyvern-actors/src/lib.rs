pub trait Actor {
    fn handle_messages(self) -> impl Future<Output = ()> + Send + Sync;
}
