use wyvern_mc::server::builder::ServerBuilder;

#[tokio::main]
async fn main() {
    ServerBuilder::new()
        .start()
        .await;
}