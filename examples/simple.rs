use wyvern_mc::server::builder::ServerBuilder;

#[tokio::main]
async fn main() {
    let b = ServerBuilder::new();
    b.start().await;
}