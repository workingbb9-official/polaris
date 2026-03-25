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
            let space = self.buf.free_space();

            if space.is_empty() {
                return Err(RecvError::DelimiterNotFound);
            }

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

    /// Returns # of bytes read.
    /// Could be less than 'bytes' if buffer is not large enough.
    pub async fn read_exact(&mut self, bytes: usize) -> Result<usize, RecvError> {
        let space = self.buf.free_space();

        let n = if space.len() < bytes {
            space.len()
        } else {
            bytes
        };

        match timeout(
            Duration::from_secs(self.config.timeout),
            self.stream.read(&mut space[..n]),
        ).await {
            Ok(Ok(n)) => {
                self.buf.head += n;
                Ok(n)
            }
            Ok(Err(_)) => Err(RecvError::IoError),
            Err(_) => Ok(0),
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

    pub fn free_space(&mut self) -> &mut [u8] {
        &mut self.storage[self.head..]
    }

    pub fn shift_leftovers(&mut self, finished: usize) {
        let leftover = self.head - finished;
        if leftover > 0 {
            self.storage.copy_within(finished..self.head, 0);
        }
        self.head = leftover;
    }
}
