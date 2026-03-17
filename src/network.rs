use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::time::{Duration, timeout};

pub enum RecvError {
    HeaderTooLarge,
    IoError,
}

pub struct SlidingBuffer {
    storage: Vec<u8>,
    head: usize,
}

impl SlidingBuffer {
    pub fn new(size: usize) -> Self {
        Self {
            storage: vec![0u8; size],
            head: 0,
        }
    }

    pub fn data(&self) -> &[u8] {
        &self.storage[..self.head]
    }

    pub fn available_capacity(&self) -> usize {
        self.storage.len() - self.head
    }

    /// Return space open for writing.
    pub fn write_area(&mut self) -> &mut [u8] {
        &mut self.storage[self.head..]
    }

    /// Move leftover data to start of the buffer.
    pub fn shift_leftovers(&mut self, finished: usize) {
        let leftover = self.head - finished;
        if leftover > 0 {
            self.storage.copy_within(finished..self.head, 0);
        }
        self.head = leftover;
    }
}

/// Read from a tcp stream into buffer.
///
/// # Arguments
/// * 'stream' - Mut ref to TcpStream to read from.
/// * 'buf' - Mut ref to a SlidingBuffer.
/// * 'timeout_secs' - Amount of time to wait for data.
///
/// # Returns
/// * 'Ok(n)' - # of bytes read.
/// * 'Ok(0)' - 0 bytes read or timeout.
/// * 'Err(RecvError)' - Error with receiving.
///
pub async fn get_msg(
    stream: &mut TcpStream,
    buf: &mut SlidingBuffer,
    timeout_secs: u64,
) -> Result<usize, RecvError> {
    loop {
        if buf.available_capacity() == 0 {
            return Err(RecvError::HeaderTooLarge);
        }

        let space = buf.write_area();
        let n = match timeout(Duration::from_secs(timeout_secs), stream.read(space)).await {
            Ok(Ok(0)) => return Ok(0),
            Ok(Ok(n)) => n,
            Ok(Err(_)) => return Err(RecvError::IoError),
            Err(_) => return Ok(0),
        };

        buf.head += n;

        if let Some(pos) = find_header_end(buf.data()) {
            return Ok(pos);
        }
    }
}

/// Write a message to a tcp stream.
///
/// # Arguments
/// * 'msg' - Binary slice to send.
/// * 'stream' - Mut ref to TcpStream to write to.
///
/// # Returns
/// * 'Ok(())' - Message sent successfully.
/// * 'Err(_)' - Failed to write or flush stream.
///
pub async fn send_msg(msg: &[u8], stream: &mut TcpStream) -> tokio::io::Result<()> {
    stream.write_all(msg).await?;
    stream.flush().await?;
    Ok(())
}

fn find_header_end(buf: &[u8]) -> Option<usize> {
    buf.windows(4).position(|w| w == b"\r\n\r\n").map(|i| i + 4)
}
