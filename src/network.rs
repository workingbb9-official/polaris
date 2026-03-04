use tokio::io::Result;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::time::{Duration, timeout};

const BUF_SIZE: usize = 8192;

pub async fn get_msg(stream: &mut TcpStream) -> Result<Option<Vec<u8>>> {
    let mut buf = Vec::with_capacity(BUF_SIZE);
    let mut tmp = [0u8; 1024];

    loop {
        let result = timeout(Duration::from_secs(5), stream.read(&mut tmp)).await;

        match result {
            Ok(Ok(0)) => return Ok(None),
            Ok(Ok(n)) => {
                buf.extend_from_slice(&tmp[..n]);
                if find_header_end(&buf).is_some() {
                    return Ok(Some(buf));
                }

                if buf.len() > BUF_SIZE {
                    return Err(std::io::Error::other("Header too large"));
                }
            }
            Ok(Err(e)) => return Err(e),
            Err(_) => return Ok(None),
        };
    }
}

pub async fn send_msg(msg: &[u8], stream: &mut TcpStream) -> Result<()> {
    stream.write_all(msg).await?;
    stream.flush().await?;
    Ok(())
}

fn find_header_end(buf: &[u8]) -> Option<usize> {
    buf.windows(4).position(|w| w == b"\r\n\r\n").map(|i| i + 4)
}
