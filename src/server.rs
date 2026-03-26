use crate::network::{Network, NetworkConfig, ReadResult};
use crate::protocol::Framing;
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
    protocol: P,
    router: Router,
}

impl<P: Protocol + std::marker::Sync + 'static> Server<P> {
    pub async fn new(addr: &str, protocol: P, router: Router) -> tokio::io::Result<Self> {
        let sock: SocketAddr = addr.parse().expect("Invalid address");
        let listener = TcpListener::bind(sock).await?;

        Ok(Self {
            listener,
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

    async fn handle_connection(&self, stream: TcpStream) {
        info!("Connected to client");
        let config = NetworkConfig::new(TIMEOUT_LEN, BUF_SIZE);
        let network = Network::new(stream, config);

        self.connection_loop(network).await;
        info!("Dropping connection");
    }

    async fn connection_loop(&self, mut network: Network) -> Option<()> {
        loop {
            let raw = self.net_read(&mut network).await?;
            let msg = self.protocol.parse(raw)?;
            let outcome = self.router.handle(msg);
            let response = self.protocol.serialize(outcome);
            network.write(&response).await.ok()?;
        }
    }

    async fn net_read(&self, network: &mut Network) -> Option<Vec<u8>> {
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
            ReadResult::BufferFull => (),
        };

        let data = network.data();

        match self.protocol.framing() {
            Framing::Delimiter(d) => match find_delimiter(data, d) {
                None => None,
                Some(pos) => {
                    let msg = Some(data[..pos].to_vec());
                    network.reset(pos);
                    msg
                }
            },
            Framing::ExactBytes(_) => None,
        }
    }
}

fn find_delimiter(buf: &[u8], delimiter: &[u8]) -> Option<usize> {
    let len = delimiter.len();
    buf.windows(len)
        .position(|w| w == delimiter)
        .map(|i| i + len)
}
