use super::*;

pub struct HttpProtocol;
impl Protocol for HttpProtocol {
    /// Split http request and pack into struct.
    ///
    /// Assumes GET method for now.
    fn parse_req(&self, raw: Vec<u8>) -> Option<ProtocolRequest> {
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

    /// Create a binary response header from HttpResponse.
    fn serialize_resp(&self, response: ProtocolResponse) -> Vec<u8> {
        match response {
            ProtocolResponse::Http {
                status,
                body,
                content_type,
            } => serialize_http(&status, &content_type, body),
            _ => Vec::new(),
        }
    }
}

fn serialize_http(status: &str, content_type: &str, body: Vec<u8>) -> Vec<u8> {
    let header = format!(
        "{}\r\n\
            Content-Security-Policy: default-src 'self'; script-src 'self';\r\n\
            Content-Length: {}\r\n\
            Content-Type: {}\r\n\
            Connection: keep-alive\r\n\
            \r\n",
        status,
        body.len(),
        content_type,
    );

    let mut final_response = header.into_bytes();

    // Add body after header
    final_response.extend(&body);

    final_response
}
