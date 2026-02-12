use polaris::NetworkServer;

#[tokio::main]
async fn main() {
    env_logger::init();
    let server = NetworkServer::new("127.0.0.1:8080");
    let _ = server.run().await;
}
