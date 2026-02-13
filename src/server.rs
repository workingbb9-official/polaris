use crate::handler;
use log::info;
use std::net::SocketAddr;
use tokio::net::TcpListener;

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
            let (stream, _sock) = self.listener.accept().await?;
            info!("Connected to client");

            tokio::spawn(async move {
                handler::handle_client(stream).await;
            });
        }
    }
}
