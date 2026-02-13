use log::info;
use tokio::io::Result;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

pub async fn handle_client(mut stream: TcpStream) -> Result<()> {
    let mut buf = [0u8; 1024];

    let n = match stream.read(&mut buf).await? {
        0 => {
            info!("Client disconnected");
            return Ok(());
        }
        n => n,
    };

    let has_get = n >= 3 && buf[..n].starts_with(b"GET");
    let has_http = n >= 4 && buf[..n].starts_with(b"HTTP");

    if has_get || has_http {
        handle_http(stream).await?;
    }

    Ok(())
}

async fn handle_http(mut stream: TcpStream) -> Result<()> {
    let content = "Hello from Rust";
    let response = format!(
        "HTTP/1.1 200 OK\r\n\
        Content-Length: {len}\r\n\
        Content-Type: text/plain\r\n\
        Connection: close\r\n\
        \r\n\
        {content}",
        len = content.len(),
        content = content
    );

    stream.write_all(response.as_bytes()).await?;
    stream.flush().await?;

    Ok(())
}
