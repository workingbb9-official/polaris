use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::time::{Duration, timeout};

pub struct Network {
    stream: TcpStream,
    buf: SlidingBuffer,
    config: NetworkConfig,
}

impl Network {
    pub fn new(stream: TcpStream, buf: SlidingBuffer, config: NetworkConfig) -> Self {
        Network {
            stream,
            buf,
            config,
        }
    }

    pub async fn read_until(&mut self, delimiter: &[u8]) -> Result<usize, RecvError> {
        loop {
            let capacity = self.buf.available_capacity();
            if capacity == 0 {
                return Err(RecvError::DelimiterNotFound);
            }

            let space = self.buf.write_area();
            let n = match timeout(
                Duration::from_secs(self.config.timeout),
                self.stream.read(space),
            )
            .await
            {
                Ok(Ok(0)) => return Ok(0),
                Ok(Ok(n)) => n,
                Ok(Err(_)) => return Err(RecvError::IoError),
                Err(_) => return Ok(0),
            };

            self.buf.head += n;

            if let Some(pos) = self.find_delimiter(delimiter) {
                return Ok(pos);
            }
        }
    }

    pub async fn write(&mut self, buf: &[u8]) -> tokio::io::Result<()> {
        self.stream.write_all(buf).await?;
        self.stream.flush().await?;

        Ok(())
    }

    pub fn data(&self) -> &[u8] {
        self.buf.data()
    }

    pub fn reset(&mut self, pos: usize) {
        self.buf.shift_leftovers(pos);
    }

    fn find_delimiter(&self, delimiter: &[u8]) -> Option<usize> {
        let len = delimiter.len();
        self.buf
            .data()
            .windows(len)
            .position(|w| w == delimiter)
            .map(|i| i + len)
    }
}

pub enum RecvError {
    DelimiterNotFound,
    IoError,
}

pub struct NetworkConfig {
    timeout: u64,
}

impl NetworkConfig {
    pub fn new(timeout: u64) -> Self {
        NetworkConfig { timeout }
    }
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
