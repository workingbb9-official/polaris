use super::*;
use crate::network::{Network, RecvError};

pub struct HttpProtocol;
impl Protocol for HttpProtocol {
    async fn read(&self, network: &mut Network) -> Option<Vec<u8>> {
        let pos = match network.read_until(b"\r\n\r\n").await {
            Err(RecvError::DelimiterNotFound) => {
                info!("HTTP header too large");
                return None;
            }
            Err(RecvError::IoError) => {
                info!("IO error when reading");
                return None;
            }
            Ok(0) => {
                info!("No data received");
                return None;
            }
            Ok(n) => n,
        };

        let data = network.data()[..pos].to_vec();
        network.reset(pos);

        Some(data)
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
