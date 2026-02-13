use tokio::io::Result;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

pub async fn get_msg(stream: &mut TcpStream) -> Result<Option<String>> {
    let mut buf = [0u8; 1024];

    let n = match stream.read(&mut buf).await? {
        0 => return Ok(None),
        n => n,
    };

    let msg = String::from_utf8_lossy(&buf[..n]).to_string();
    Ok(Some(msg))
}

pub async fn send_msg(msg: &str, stream: &mut TcpStream) -> Result<()> {
    stream.write_all(msg.as_bytes()).await?;
    stream.flush().await?;
    Ok(())
}
