use crate::network::{Network, NetworkConfig, ReadResult};
use crate::protocol::Framing;
use crate::protocol::Protocol;

use log::{info, warn};
use std::sync::Arc;

use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream};

pub struct Server<P: Protocol> {
    listener: TcpListener,
    config: NetworkConfig,
    protocol: P,
}

impl<P: Protocol + std::marker::Sync + 'static> Server<P> {
    pub async fn new(addr: &str, config: NetworkConfig, protocol: P) -> tokio::io::Result<Self> {
        let sock: SocketAddr = addr.parse().expect("Invalid address");
        let listener = TcpListener::bind(sock).await?;

        Ok(Self {
            listener,
            config,
            protocol,
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
                None => todo!(),
            };
            let outcome = self.protocol.route(msg);
            let response = self.protocol.serialize(outcome);
            network.write(&response).await.ok()?;
        }
    }

    async fn net_read(&self, network: &mut Network) -> Option<Vec<u8>> {
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
                    if let Some((vec, pos)) = handle_frame(&self.protocol.framing(), network.data())
                    {
                        network.reset(pos);
                        return Some(vec);
                    }

                    info!("Buffer full, frame not found");
                    return None;
                }
                ReadResult::Data => {
                    if let Some((vec, pos)) = handle_frame(&self.protocol.framing(), network.data())
                    {
                        network.reset(pos);
                        return Some(vec);
                    }
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

// Returns message and position where it ended.
fn handle_frame(framing: &Framing, buf: &[u8]) -> Option<(Vec<u8>, usize)> {
    match framing {
        Framing::Delimiter(d) => {
            let idx = find_delimiter(buf, d)?;
            let len = idx.saturating_sub(d.len());
            Some((buf[..len].to_vec(), idx))
        }
        Framing::ExactBytes(n) => {
            if buf.len() < *n {
                return None;
            }

            Some((buf[..*n].to_vec(), *n))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn find_delimiter_in_middle_returns_index() {
        let buf: &[u8] = b"find$%_delimiter_inthis^&";
        let result = find_delimiter(buf, b"delimiter");
        assert_eq!(result, Some(16));
    }

    #[test]
    fn find_delimiter_at_start_returns_index() {
        let buf: &[u8] = b"delimiter@$_start";
        let result = find_delimiter(buf, b"delimiter");
        assert_eq!(result, Some(9));
    }

    #[test]
    fn find_delimiter_at_end_returns_index() {
        let buf: &[u8] = b"@TheEnd$is_thedelimiter";
        let result = find_delimiter(buf, b"delimiter");
        assert_eq!(result, Some(23));
    }

    #[test]
    fn find_delimiter_not_found_returns_none() {
        let buf: &[u8] = b"$oDelimInThis*ne";
        let result = find_delimiter(buf, b"delimiter");
        assert_eq!(result, None);
    }

    #[test]
    fn find_delimiter_empty_buffer_returns_none() {
        let buf: &[u8] = b"";
        let result = find_delimiter(buf, b"delimiter");
        assert_eq!(result, None);
    }

    fn dummy_handler(_: &[u8]) -> ProtocolResponse {
        HttpResponse::FileFound {
            content_type: "text/plain".to_string(),
            body: b"hello".to_vec(),
        }
    }

    #[test]
    fn router_valid_route_returns_found() {
        let mut router = Router::new();
        router.add_route(b"/", dummy_handler);

        let request = HttpMessage {
            method: "GET".to_string(),
            path: "/".to_string(),
            body: Vec::new(),
        };

        let response = router.handle(request);
        assert_eq!(
            response,
            HttpResponse::FileFound {
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

    #[test]
    fn delimiter_framing_returns_pos() {
        let buf = b"HttpMessage\r\n\r\nMoreStuff";

        let result = handle_frame(&Framing::Delimiter(b"\r\n\r\n"), buf);
        assert_eq!(result, Some((buf[..11].to_vec(), 15)));
    }

    #[test]
    fn delimiter_framing_no_delimiter() {
        let buf = b"ThereIsNoDelimiter";

        let result = handle_frame(&Framing::Delimiter(b"\r\n\r\n"), buf);
        assert_eq!(result, None);
    }

    #[test]
    fn exact_bytes_framing_returns_bytes() {
        let buf = b"ThisIs17BytesLong";

        let result = handle_frame(&Framing::ExactBytes(13), buf);
        assert_eq!(result, Some((buf[..13].to_vec(), 13)));
    }

    #[test]
    fn exact_bytes_framing_buffer_too_short() {
        let buf = b"ShortString18Bytes";

        let result = handle_frame(&Framing::ExactBytes(20), buf);
        assert_eq!(result, None);
    }
}
