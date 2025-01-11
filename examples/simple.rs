use wyvern_mc::server::builder::ServerBuilder;

#[tokio::main]
async fn main() {
    let mut b = ServerBuilder::new();
    b.add_system(some_system);
    b.start().await;
}

async fn some_system() {
    println!("System is executing!");
}