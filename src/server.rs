use crate::handler;
use log::info;
use std::net::SocketAddr;
use tokio::io::AsyncReadExt;
use tokio::net::{TcpListener, TcpStream};

pub struct NetworkServer {
    listener: TcpListener,
}

impl NetworkServer {
    pub async fn new(addr: &str) -> tokio::io::Result<Self> {
        let sock: SocketAddr = addr.parse().expect("Invalid address");
        let listener = TcpListener::bind(sock).await?;
        Ok(Self { listener })
    }

    pub async fn run(&self) -> tokio::io::Result<()> {
        loop {
            let (stream, _) = self.listener.accept().await?;
            info!("Connected to client");

            tokio::spawn(async move {
                let _ = handler::handle_client(stream).await;
            });
        }
    }

    async fn get_msg(stream: &mut TcpStream) -> tokio::io::Result<Option<String>> {
        let mut buf = [0u8; 1024];

        let n = match stream.read(&mut buf).await? {
            0 => {
                info!("Client disconnected");
                return Ok(None);
            }
            n => n,
        };

        let msg = String::from_utf8_lossy(&buf[..n]).to_string();
        Ok(Some(msg))
    }
}
