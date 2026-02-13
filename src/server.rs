use crate::handler;
use crate::network;
use log::{info, warn};

use std::net::SocketAddr;
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
            Self::handle_connection(stream).await;
        }
    }

    async fn handle_connection(mut stream: TcpStream) {
        tokio::spawn(async move {
            let msg = match network::get_msg(&mut stream).await {
                Ok(Some(msg)) => msg,
                Ok(None) => {
                    info!("Client disconnected");
                    return;
                }
                Err(e) => {
                    warn!("Failed to get msg with error: {}", e);
                    return;
                }
            };
            let response = handler::handle_client(&msg).await;
            if let Err(e) = network::send_msg(&response, &mut stream).await {
                warn!("Failed to send msg with error: {}", e);
            }
        });
    }
}
