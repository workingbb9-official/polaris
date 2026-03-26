use super::*;
use crate::network::{Network, RecvResult};

pub struct HttpProtocol;
impl Protocol for HttpProtocol {
    async fn read(&self, network: &mut Network) -> Option<Vec<u8>> {
        match network.read().await {
            RecvResult::NoData => {
                info!("Received no data");
                return None;
            }
            RecvResult::Timeout => {
                info!("Connection timed out");
                return None;
            }
            RecvResult::IoError => {
                info!("IO error when reading");
                return None;
            }
            RecvResult::BufferFull => (),
        };

        if let Some(pos) = find_delimiter(network.data()) {
            let data = network.data()[..pos].to_vec();
            network.reset(pos);
            Some(data)
        } else {
            None
        }
    }

    fn parse(&self, raw: Vec<u8>) -> Option<ProtocolRequest> {
        let request = String::from_utf8(raw).ok()?;

        let first_line = request.lines().next()?;
        let mut parts = first_line.split_whitespace();

        let method = parts.next()?;
        let path = parts.next()?;

        let http_req = ProtocolRequest::Http {
            method: method.to_string(),
            path: path.to_string(),
            body: Vec::new(),
        };

        Some(http_req)
    }

    fn serialize(&self, response: ProtocolResponse) -> Vec<u8> {
        match response {
            ProtocolResponse::FileFound { content_type, body } => {
                serialize_http("HTTP/1.1 200 OK", &content_type, "keep-alive", body)
            }
            ProtocolResponse::FileNotFound => serialize_http(
                "HTTP/1.1 404 Not Found",
                "keep-alive",
                "text/plain",
                b"Polaris\nFile Not Found".to_vec(),
            ),
            ProtocolResponse::BadRequest => serialize_http(
                "HTTP/1.1 400 Bad Request",
                "text/plain",
                "close",
                b"Polaris\nBad Request".to_vec(),
            ),
        }
    }
}

fn serialize_http(status: &str, content_type: &str, conn: &str, body: Vec<u8>) -> Vec<u8> {
    let header = format!(
        "{}\r\n\
            Content-Security-Policy: default-src 'self'; script-src 'self';\r\n\
            Content-Length: {}\r\n\
            Content-Type: {}\r\n\
            Connection: {}\r\n\
            \r\n",
        status,
        body.len(),
        content_type,
        conn,
    );

    let mut final_response = header.into_bytes();

    // Add body after header
    final_response.extend(&body);

    final_response
}

fn find_delimiter(buf: &[u8]) -> Option<usize> {
    buf.windows(4).position(|w| w == b"\r\n\r\n").map(|i| i + 4)
}
