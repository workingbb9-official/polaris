use tokio::io::Result;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::time::{Duration, timeout};

pub async fn get_msg(stream: &mut TcpStream) -> Result<Option<Vec<u8>>> {
    let mut buf = [0u8; 1024];

    let result = timeout(Duration::from_secs(5), stream.read(&mut buf)).await;

    match result {
        Ok(Ok(0)) => Ok(None),
        Ok(Ok(_)) => Ok(Some(buf.to_vec())),
        Ok(Err(e)) => Err(e),
        Err(_) => Ok(None),
    }
}

pub async fn send_msg(msg: &[u8], stream: &mut TcpStream) -> Result<()> {
    stream.write_all(msg).await?;
    stream.flush().await?;
    Ok(())
}
