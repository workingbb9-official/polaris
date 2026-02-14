use tokio::io::Result;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

pub async fn get_msg(stream: &mut TcpStream) -> Result<Option<Vec<u8>>> {
    let mut buf = [0u8; 1024];

    if stream.read(&mut buf).await? == 0 {
        return Ok(None);
    }

    Ok(Some(buf.to_vec()))
}

pub async fn send_msg(msg: &[u8], stream: &mut TcpStream) -> Result<()> {
    stream.write_all(msg).await?;
    stream.flush().await?;
    Ok(())
}
