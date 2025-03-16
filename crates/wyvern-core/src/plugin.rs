use crate::server::ServerBuilder;

pub trait Plugin {
    fn build(&self, builder: ServerBuilder) -> ServerBuilder;
}
