use tokio::io::Result;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::time::{Duration, timeout};

pub enum RecvError {
    HeaderTooLarge,
    IoError,
}

/// Reads from a tcp stream into buffer.
///
/// # Arguments
/// * 'stream' - Mut ref to TcpStream to read from.
/// * 'buf' - Mut ref to a buffer that stores what is read.
/// * 'max_size' - Maximum bytes to read into buffer.
/// * 'timeout_secs' - Amount of time to wait for data.
///
/// # Returns
/// * 'Ok(n)' - # of bytes read.
/// * 'Ok(0)' - 0 bytes read or timeout.
/// * 'Err(RecvError)' - Error with receiving.
///
pub async fn get_msg(
    stream: &mut TcpStream,
    buf: &mut Vec<u8>,
    max_size: usize,
    timeout_secs: u64,
) -> std::result::Result<usize, RecvError> {
    let mut tmp = [0u8; 1024];
    buf.clear();

    loop {
        let result = timeout(Duration::from_secs(timeout_secs), stream.read(&mut tmp)).await;

        match result {
            Ok(Ok(n)) => {
                if n == 0 {
                    return Ok(0);
                }

                if buf.len() + n > max_size {
                    return Err(RecvError::HeaderTooLarge);
                }

                buf.extend_from_slice(&tmp[..n]);

                if find_header_end(buf).is_some() {
                    return Ok(buf.len());
                }
            }
            Ok(Err(_)) => return Err(RecvError::IoError),
            Err(_) => return Ok(0),
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
