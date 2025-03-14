pub trait Actor {
    fn handle_messages(&mut self);
}

pub type ActorResult<T> = Result<T, ActorError>;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum ActorError {
    ActorDoesNotExist,
    ActorIsNotLoaded,
    IndexOutOfBounds,
    BadRequest,
    ComponentNotFound,
    ActorHasBeenDropped,
}
