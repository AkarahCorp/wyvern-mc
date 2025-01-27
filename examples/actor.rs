use wyvern_mc::actors::Actor;

#[wyvern_mc::actor(Person, PersonMessage)]
struct RawPerson {
    name: String,
    age: i32,
}

#[wyvern_mc::message(Person, PersonMessage)]
impl RawPerson {
    #[GetName]
    pub async fn name(&self) -> String {
        self.name.clone()
    }

    #[GetAge]
    pub async fn age(&self) -> i32 {
        self.age
    }

    #[SetAge]
    pub async fn set_age(&mut self, new_age: i32) {
        self.age = new_age;
    }
}

#[tokio::main]
async fn main() {
    let (sender, receiver) = tokio::sync::mpsc::channel(128);
    let mut p = RawPerson {
        name: "John".to_string(),
        age: 35,
        receiver,
    };
    tokio::spawn(async move {
        loop {
            p.handle_messages().await;
        }
    });

    let p = Person { sender };
    p.set_age(17).await;

    loop {}
}
