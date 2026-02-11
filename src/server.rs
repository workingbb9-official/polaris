use tokio::net::{TcpListener};
use log::{info};
use std::net::SocketAddr;
use crate::handler;

pub struct NetworkServer {
    addr: SocketAddr
}

impl NetworkServer {
    pub fn new(addr_str: &str) -> Self {
        info!("Connecting to {}", addr_str);
        let a: SocketAddr = addr_str.parse().expect("Invalid address");
        Self{addr: a}
    }

    pub async fn run(&self) -> tokio::io::Result<()> {
        let listener: TcpListener = TcpListener::bind(self.addr).await?;

        loop {
            let (stream, _sock) = listener.accept().await?;
            info!("Connected to client");

            tokio::spawn(async move {
                handler::handle_client(stream).await;
            });
        }
    }
}
