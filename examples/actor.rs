use tokio::sync::oneshot::Sender;
use wyvern_mc::actors::Actor;

pub enum PersonMessage {
    GetName(Sender<String>),
}

#[wyvern_mc::actor(Person, PersonMessage)]
struct RawPerson {
    name: String,
}

#[wyvern_mc::message(Person, PersonMessage)]
impl RawPerson {
    #[GetName]
    pub async fn name(&self) -> String {
        self.name.clone()
    }
}

#[tokio::main]
async fn main() {
    let (sender, receiver) = tokio::sync::mpsc::channel(128);
    let p = RawPerson {
        name: "Endistic".to_string(),
        receiver,
    };
    tokio::spawn(Actor::handle_messages(p));

    let p = Person { sender };
    assert_eq!(p.name().await, "Endistic".to_string());
}
