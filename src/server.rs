use crate::network::{Network, NetworkConfig, ReadResult};
use crate::protocol::Framing;
use crate::protocol::{Protocol, ProtocolRequest, ProtocolResponse};

use log::{info, warn};
use std::collections::HashMap;
use std::sync::Arc;

use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream};

type Handler = fn(&[u8]) -> ProtocolResponse;

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
    pub fn handle(&self, msg: ProtocolRequest) -> ProtocolResponse {
        match msg {
            ProtocolRequest::Http { path, .. } => {
                let raw = path.into_bytes();
                match self.routes.get(&raw) {
                    Some(handler) => handler(&raw),
                    None => ProtocolResponse::FileNotFound,
                }
            }
            ProtocolRequest::Raw(raw) => match self.routes.get(&raw) {
                Some(handler) => handler(&raw),
                None => ProtocolResponse::FileNotFound,
            },
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
    config: NetworkConfig,
    protocol: P,
    router: Router,
}

impl<P: Protocol + std::marker::Sync + 'static> Server<P> {
    pub async fn new(
        addr: &str,
        config: NetworkConfig,
        protocol: P,
        router: Router,
    ) -> tokio::io::Result<Self> {
        let sock: SocketAddr = addr.parse().expect("Invalid address");
        let listener = TcpListener::bind(sock).await?;

        Ok(Self {
            listener,
            config,
            protocol,
            router,
        })
    }

    /// Connect to clients and spawn a task.
    pub async fn run(self: Arc<Self>) -> tokio::io::Result<()> {
        loop {
            let (stream, _) = self.listener.accept().await?;

            let server_ptr = Arc::clone(&self);
            tokio::spawn(async move {
                server_ptr.handle_connection(stream).await;
            });
        }
    }

    pub fn local_addr(&self) -> SocketAddr {
        self.listener.local_addr().unwrap()
    }

    async fn handle_connection(&self, stream: TcpStream) {
        info!("Connected to client");
        let network = Network::new(stream, self.config);

        self.connection_loop(network).await;
        info!("Dropping connection");
    }

    async fn connection_loop(&self, mut network: Network) -> Option<()> {
        loop {
            let raw = self.net_read(&mut network).await?;
            let msg = match self.protocol.parse(raw) {
                Some(msg) => msg,
                None => {
                    let bytes = self.protocol.serialize(ProtocolResponse::BadRequest);
                    network.write(&bytes).await.ok()?;
                    return None;
                }
            };
            let outcome = self.router.handle(msg);
            let response = self.protocol.serialize(outcome);
            network.write(&response).await.ok()?;
        }
    }

    async fn net_read(&self, network: &mut Network) -> Option<Vec<u8>> {
        let Framing::Delimiter(d) = self.protocol.framing() else {
            return None;
        };

        loop {
            match network.read().await {
                ReadResult::NoData => {
                    info!("Received no data");
                    return None;
                }
                ReadResult::Timeout => {
                    info!("Connection timed out");
                    return None;
                }
                ReadResult::IoError => {
                    warn!("IO error when reading");
                    return None;
                }
                ReadResult::BufferFull => {
                    let pos = find_delimiter(network.data(), d)?;
                    let msg = network.data()[..pos].to_vec();
                    network.reset(pos);
                    return Some(msg);
                }

                ReadResult::Data => {
                    if let Some(pos) = find_delimiter(network.data(), d) {
                        let msg = network.data()[..pos].to_vec();
                        network.reset(pos);
                        return Some(msg);
                    };
                }
            };
        }
    }
}

// Returns index directly after delimiter.
fn find_delimiter(buf: &[u8], delimiter: &[u8]) -> Option<usize> {
    let len = delimiter.len();
    buf.windows(len)
        .position(|w| w == delimiter)
        .map(|i| i + len)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn find_delimiter_in_middle_returns_index() {
        let buf: Vec<u8> = b"find$%_delimiter_inthis^&".to_vec();
        let result = find_delimiter(&buf, b"delimiter");
        assert_eq!(result, Some(16));
    }

    #[test]
    fn find_delimiter_at_start_returns_index() {
        let buf: Vec<u8> = b"delimiter@$_start".to_vec();
        let result = find_delimiter(&buf, b"delimiter");
        assert_eq!(result, Some(9));
    }

    #[test]
    fn find_delimiter_at_end_returns_index() {
        let buf: Vec<u8> = b"@TheEnd$is_thedelimiter".to_vec();
        let result = find_delimiter(&buf, b"delimiter");
        assert_eq!(result, Some(23));
    }

    #[test]
    fn find_delimiter_not_found_returns_none() {
        let buf: Vec<u8> = b"$oDelimInThis*ne".to_vec();
        let result = find_delimiter(&buf, b"delimiter");
        assert_eq!(result, None);
    }

    #[test]
    fn find_delimiter_empty_buffer_returns_none() {
        let buf = Vec::new();
        let result = find_delimiter(&buf, b"delimiter");
        assert_eq!(result, None);
    }

    fn dummy_handler(_: &[u8]) -> ProtocolResponse {
        ProtocolResponse::FileFound {
            content_type: "text/plain".to_string(),
            body: b"hello".to_vec(),
        }
    }

    #[test]
    fn router_valid_route_returns_found() {
        let mut router = Router::new();
        router.add_route(b"/", dummy_handler);

        let request = ProtocolRequest::Http {
            method: "GET".to_string(),
            path: "/".to_string(),
            body: Vec::new(),
        };

        let response = router.handle(request);
        assert_eq!(
            response,
            ProtocolResponse::FileFound {
                content_type: "text/plain".to_string(),
                body: b"hello".to_vec(),
            }
        );
    }

    #[test]
    fn router_invalid_route_returns_not_found() {
        let mut router = Router::new();
        router.add_route(b"/", dummy_handler);

        let request = ProtocolRequest::Http {
            method: "GET".to_string(),
            path: "/fake".to_string(),
            body: Vec::new(),
        };

        let response = router.handle(request);
        assert_eq!(response, ProtocolResponse::FileNotFound);
    }
}
