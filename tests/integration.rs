use polaris::{HttpProtocol, NetworkConfig, ProtocolResponse, Router, Server};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

async fn spawn_test_server() -> SocketAddr {
    let config = NetworkConfig::new(Duration::from_millis(100), 8192);
    let protocol = HttpProtocol;

    let mut router = Router::new();
    router.add_route(b"/", |_| ProtocolResponse::FileFound {
        content_type: "text/plain".to_string(),
        body: b"hello".to_vec(),
    });

    let server = Server::new("127.0.0.1:0", config, protocol, router)
        .await
        .expect("Failed to create server");

    let addr = server.local_addr();

    tokio::spawn(async move {
        Arc::new(server).run().await.unwrap();
    });

    addr
}

#[tokio::test]
async fn get_known_route_returns_200() {
    let addr = spawn_test_server().await;

    let mut stream = TcpStream::connect(addr).await.unwrap();
    stream.write_all(b"GET / HTTP/1.1\r\n\r\n").await.unwrap();

    let mut buf = vec![0u8; 1024];
    let n = stream.read(&mut buf).await.unwrap();
    let response = String::from_utf8_lossy(&buf[..n]);

    assert!(response.contains("200 OK"));
    assert!(response.contains("hello"));
}

#[tokio::test]
async fn get_unknown_route_returns_404() {
    let addr = spawn_test_server().await;

    let mut stream = TcpStream::connect(addr).await.unwrap();
    stream
        .write_all(b"GET /fake HTTP/1.1\r\n\r\n")
        .await
        .unwrap();

    let mut buf = vec![0u8; 1024];
    let n = stream.read(&mut buf).await.unwrap();
    let response = String::from_utf8_lossy(&buf[..n]);

    assert!(response.contains("404 Not Found"));
}

#[tokio::test]
async fn bad_request_drops_connection() {
    let addr = spawn_test_server().await;

    let mut stream = TcpStream::connect(addr).await.unwrap();
    stream
        .write_all(b"GIVE&ME$THE^PASSWORD\r\n\r\n")
        .await
        .unwrap();

    let mut buf = vec![0u8; 1024];
    let mut n = stream.read(&mut buf).await.unwrap();
    let response = String::from_utf8_lossy(&buf[..n]);

    assert!(response.contains("Connection: close"));

    buf.clear();
    n = stream.read(&mut buf).await.unwrap();
    assert_eq!(n, 0);
}

#[tokio::test]
async fn no_delimiter_times_out() {
    let addr = spawn_test_server().await;

    let mut stream = TcpStream::connect(addr).await.unwrap();
    stream.write_all(b"WastingYourTime\r\n").await.unwrap();

    let mut buf = vec![0u8; 1024];
    let n = stream.read(&mut buf).await.unwrap();

    assert_eq!(n, 0);
}
