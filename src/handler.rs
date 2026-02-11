use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use log::{info, warn};

pub async fn handle_client(mut stream: TcpStream) {
    let mut buf = [0u8; 1024];

    let n = match stream.read(&mut buf).await {
        Err(e) => {
            warn!("Received error {}", e);
            return;
        },
        Ok(0) => {
            info!("Client disconnected");
            return;
        },
        Ok(n) => n,
    };
    
    if n >= 3 && buf[..n].starts_with(b"GET") {
        handle_http(stream).await;
    } else if n >= 4 && buf[..n].starts_with(b"HTTP") {
        handle_http(stream).await;
    }
}

async fn handle_http(mut stream: TcpStream) {
    let content = "Hello from Rust";
    let response = format!(
                    "HTTP/1.1 200 OK\r\n\
                     Content-Length: {}\r\n\
                     Content-Type: text/plain\r\n\
                     Connection: close\r\n\
                     \r\n\
                     {}",
                    content.len(),
                    content);
    
    let _ = stream.write_all(response.as_bytes()).await;
    let _ = stream.flush().await;
}
