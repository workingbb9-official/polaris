use crate::network;
use crate::protocol::Protocol;

use log::{info, warn};
use std::collections::HashMap;
use std::sync::Arc;

use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream};

type Handler = fn(&[u8]) -> Vec<u8>;

#[derive(Default)]
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

    pub fn handle(&self, path: &[u8]) -> Vec<u8> {
        match self.routes.get(path) {
            Some(handler) => handler(path),
            None => "No URL".into(),
        }
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

    async fn handle_connection(&self, mut stream: TcpStream) {
        loop {
            let msg = match network::get_msg(&mut stream).await {
                Ok(Some(msg)) => msg,
                Ok(None) => {
                    info!("Dropping socket");
                    break;
                }
                Err(e) => {
                    warn!("Failed to get msg with error: {}", e);
                    continue;
                }
            };

            let p_msg = match self.protocol.parse(&msg) {
                Some(p) => p,
                None => {
                    warn!("Failed to parse msg");
                    let bad_req = b"HTTP/1.1 400 Bad Request\r\n\
                                   Connection: close\r\n\r\n";

                    if let Err(e) = network::send_msg(bad_req, &mut stream).await {
                        warn!("Failed to send msg with error: {}", e);
                    }
                    break;
                }
            };

            let resp = self.router.handle(&p_msg);
            let f_resp = self.protocol.format(&resp);

            if let Err(e) = network::send_msg(&f_resp, &mut stream).await {
                warn!("Failed to send msg with error: {}", e);
            }
        }
    }
}
