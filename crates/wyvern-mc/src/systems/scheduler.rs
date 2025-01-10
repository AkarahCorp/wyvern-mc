use super::{intos::IntoSystem, system::System};

pub struct Scheduler {
    systems: Vec<StoredSystem>
}

impl Scheduler {
    pub fn add_system<I, S: System + 'static>(&mut self, system: impl IntoSystem<I, System = S>) {
        self.systems.push(Box::new(system.into_system()));
    }
}

pub type StoredSystem = Box<dyn System>;