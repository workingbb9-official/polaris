use crate::network;
use crate::protocol::{Protocol, ProtocolRequest, ProtocolResponse};

use log::{info, warn};
use std::collections::HashMap;
use std::sync::Arc;

use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream};

type Handler = fn(&[u8]) -> ProtocolResponse;

const BUF_SIZE: usize = 8192;
const TIMEOUT_LEN: u64 = 5;

/// Use to map a path to an action and response.
pub struct Router {
    routes: HashMap<Vec<u8>, Handler>,
}

impl Router {
    pub fn new() -> Self {
        Self {
            routes: HashMap::new(),
        }
    }

    pub fn add_route(&mut self, path: &[u8], handler: Handler) {
        self.routes.insert(path.into(), handler);
    }

    /// Map a path to given handler.
    pub fn handle(&self, path: &[u8]) -> ProtocolResponse {
        match self.routes.get(path) {
            Some(handler) => handler(path),
            None => ProtocolResponse::FileNotFound,
        }
    }
}

impl Default for Router {
    fn default() -> Self {
        Self::new()
    }
}

pub struct Server<P: Protocol> {
    listener: TcpListener,
    protocol: P,
    router: Router,
}

impl<P: Protocol + std::marker::Sync + std::marker::Send + 'static> Server<P> {
    pub async fn new(addr: &str, protocol: P, router: Router) -> tokio::io::Result<Self> {
        let sock: SocketAddr = addr.parse().expect("Invalid address");
        let listener = TcpListener::bind(sock).await?;

        Ok(Self {
            listener,
            protocol,
            router,
        })
    }

    /// Connect to client and spawn a task.
    pub async fn run(self: Arc<Self>) -> tokio::io::Result<()> {
        loop {
            let (stream, _) = self.listener.accept().await?;
            info!("Connected to client");

            let server_ptr = Arc::clone(&self);
            tokio::spawn(async move {
                server_ptr.handle_connection(stream).await;
            });
        }
    }

    /// Handle a connection task for a single client.
    ///
    /// Receive bytes, parse, handle, format and send response.
    ///
    async fn handle_connection(&self, stream: TcpStream) {
        let buf = network::SlidingBuffer::new(BUF_SIZE);
        let config = network::NetworkConfig::new(TIMEOUT_LEN);
        let mut network = network::Network::new(stream, buf, config);

        loop {
            // Receive from socket
            let pos = match network.read_until(b"\r\n\r\n").await {
                Err(network::RecvError::DelimiterNotFound) => {
                    info!("Header too large");
                    return;
                }
                Err(network::RecvError::IoError) => {
                    info!("Sys error with receiving");
                    return;
                }
                Ok(0) => {
                    info!("No data, dropping socket");
                    return;
                }
                Ok(n) => n,
            };

            // Parse received data
            let raw = &network.data()[..pos];
            let request = match self.protocol.parse_req(raw.to_vec()) {
                Some(p) => p,
                None => {
                    warn!("Failed to parse msg");
                    let bad_resp = ProtocolResponse::BadRequest;
                    let raw_resp = self.protocol.serialize_resp(bad_resp);

                    if let Err(e) = network.write(&raw_resp).await {
                        warn!("Failed to send msg with error: {}", e);
                    }

                    return;
                }
            };

            // Look up handler and format response
            let resp = match request {
                ProtocolRequest::Http { path, .. } => {
                    let path = path.as_bytes();
                    self.router.handle(path)
                }
                ProtocolRequest::Raw(vec) => self.router.handle(&vec),
            };

            let raw_resp = self.protocol.serialize_resp(resp);

            // Send response
            if let Err(e) = network.write(&raw_resp).await {
                warn!("Failed to send msg with error: {}", e);
            }

            // Reset network for next read
            network.reset(pos);
        }
    }
}
