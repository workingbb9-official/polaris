use log::warn;
use polaris::NetworkServer;

#[tokio::main]
async fn main() {
    env_logger::init();
    let port = "127.0.0.1:8080";
    let server = NetworkServer::new(port).await.expect("Failed to bind");
    if let Err(e) = server.run().await {
        warn!("Failed to accept with error: {}", e);
    }
}
