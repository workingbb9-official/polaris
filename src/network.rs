use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::time::{Duration, timeout};

use std::num::NonZeroUsize;

pub struct Network {
    stream: TcpStream,
    buf: NetworkBuffer,
    config: NetworkConfig,
}

impl Network {
    pub fn new(stream: TcpStream, config: NetworkConfig) -> Self {
        Network {
            stream,
            buf: NetworkBuffer::new(config.buf_size),
            config,
        }
    }

    pub async fn read(&mut self) -> ReadResult {
        let n = match timeout(
            Duration::from_secs(self.config.timeout),
            self.stream.read(&mut self.buf.storage),
        )
        .await
        {
            Ok(Err(_)) => return ReadResult::IoError,
            Err(_) => return ReadResult::Timeout,
            Ok(Ok(0)) => return ReadResult::NoData,
            Ok(Ok(n)) => n,
        };

        if n == self.buf.storage.len() {
            return ReadResult::BufferFull;
        }

        self.buf.filled += n;
        ReadResult::Data
    }

    pub async fn write(&mut self, buf: &[u8]) -> tokio::io::Result<()> {
        self.stream.write_all(buf).await?;
        self.stream.flush().await?;

        Ok(())
    }

    pub fn data(&self) -> &[u8] {
        &self.buf.storage
    }

    pub fn reset(&mut self, pos: usize) {
        self.buf.shift(pos);
    }
}

pub enum ReadResult {
    IoError,
    NoData,
    Timeout,
    BufferFull,
    Data,
}

#[derive(Copy, Clone)]
pub struct NetworkConfig {
    timeout: u64,
    buf_size: NonZeroUsize,
}

impl NetworkConfig {
    pub fn new(timeout: u64, buf_size: NonZeroUsize) -> Self {
        NetworkConfig { timeout, buf_size }
    }
}

struct NetworkBuffer {
    storage: Vec<u8>,
    filled: usize,
}

impl NetworkBuffer {
    fn new(size: NonZeroUsize) -> Self {
        Self {
            storage: vec![0u8; size.into()],
            filled: 0,
        }
    }

    fn shift(&mut self, pos: usize) {
        self.storage.copy_within(pos.., 0);
        self.filled = pos;
    }
}
