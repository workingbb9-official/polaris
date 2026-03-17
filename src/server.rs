use crate::network;
use crate::protocol::{HttpResponse, Protocol};

use log::{info, warn};
use std::collections::HashMap;
use std::sync::Arc;

use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream};

type Handler = fn(&[u8]) -> HttpResponse;

static DEFAULT_HTML_NOT_FOUND: &str = r#"<!DOCTYPE html>
    <html>
        <head><title>Polaris</title></head>
        <body>
            <h1>404 Not Found</h1>
        </body>
    </html>
    "#;

const BUF_SIZE: usize = 8192;
const TIMEOUT_LEN: u64 = 5;

/// Set default as simple html page.
fn default_err_handler(_: &[u8]) -> HttpResponse {
    let body = DEFAULT_HTML_NOT_FOUND.as_bytes().to_vec();

    HttpResponse::new(body, "text/html".to_string())
}

/// Use to map a path to an action and response.
pub struct Router {
    routes: HashMap<Vec<u8>, Handler>,
    err_handler: Handler,
}

impl Router {
    pub fn new() -> Self {
        Self {
            routes: HashMap::new(),
            err_handler: default_err_handler,
        }
    }

    pub fn add_route(&mut self, path: &[u8], handler: Handler) {
        self.routes.insert(path.into(), handler);
    }

    /// Set error handler for path not found.
    pub fn add_err_handler(&mut self, handler: Handler) {
        self.err_handler = handler;
    }

    /// Map a path to given handler.
    pub fn handle(&self, path: &[u8]) -> HttpResponse {
        match self.routes.get(path) {
            Some(handler) => handler(path),
            None => (self.err_handler)(path),
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
    async fn handle_connection(&self, mut stream: TcpStream) {
        let mut buf = network::SlidingBuffer::new(BUF_SIZE);

        loop {
            // Receive from socket
            let pos = match network::get_msg(&mut stream, &mut buf, TIMEOUT_LEN).await {
                Err(network::RecvError::HeaderTooLarge) => {
                    info!("Header too large");
                    continue;
                }
                Err(network::RecvError::IoError) => {
                    info!("Sys error with receiving");
                    continue;
                }
                Ok(0) => {
                    info!("No data, dropping socket");
                    break;
                }
                Ok(n) => n,
            };

            // Parse received data
            let p_msg = match self.protocol.parse(&buf.data()[..pos]) {
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

            // Lookup handler for path and format response
            let resp = self.router.handle(&p_msg);
            let f_resp = self.protocol.format(&resp);

            // Send message
            if let Err(e) = network::send_msg(&f_resp, &mut stream).await {
                warn!("Failed to send msg with error: {}", e);
            }

            // Move leftovers from next request to start of buffer
            buf.shift_leftovers(pos);
        }
    }
}
