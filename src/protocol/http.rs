use super::*;
use std::collections::HashMap;

pub struct HttpMessage {
    method: String,
    path: String,
    body: Vec<u8>,
}

pub enum HttpResponse {
    FileFound { content_type: String, body: Vec<u8> },
    NotFound,
    BadRequest,
}

type HttpHandler = fn(&[u8]) -> HttpResponse;

pub struct HttpProtocol {
    routes: HashMap<String, HttpHandler>,
}

impl HttpProtocol {
    pub fn new() -> Self {
        Self {
            routes: HashMap::new(),
        }
    }

    pub fn add_route(&mut self, method: &str, path: &str, handler: HttpHandler) {
        let key = format!("{} {}", method, path);
        self.routes.insert(key, handler);
    }
}

impl Default for HttpProtocol {
    fn default() -> Self {
        Self::new()
    }
}

impl Protocol for HttpProtocol {
    type Message = HttpMessage;
    type Response = HttpResponse;

    fn framing(&self) -> Framing {
        Framing::Delimiter(b"\r\n\r\n")
    }

    fn parse(&self, raw: Vec<u8>) -> Option<HttpMessage> {
        let request = String::from_utf8(raw).ok()?;

        let first_line = request.lines().next()?;
        let mut parts = first_line.split_whitespace();

        let method = parts.next()?;
        let path = parts.next()?;
        let _version = parts.next()?;

        let http_req = HttpMessage {
            method: method.to_string(),
            path: path.to_string(),
            body: Vec::new(),
        };

        Some(http_req)
    }

    fn route(&self, msg: HttpMessage) -> HttpResponse {
        let key = format!("{} {}", msg.method, msg.path);

        if let Some(handler) = self.routes.get(&key) {
            return handler(&msg.body[..]);
        }

        HttpResponse::NotFound
    }

    fn serialize(&self, response: HttpResponse) -> Vec<u8> {
        match response {
            HttpResponse::FileFound { content_type, body } => {
                serialize_http("HTTP/1.1 200 OK", &content_type, "keep-alive", body)
            }
            HttpResponse::NotFound => serialize_http(
                "HTTP/1.1 404 Not Found",
                "text/plain",
                "keep-alive",
                b"Polaris\nFile Not Found".to_vec(),
            ),
            HttpResponse::BadRequest => serialize_http(
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_valid_get_request() {
        let result = Protocol::parse(&HttpProtocol, b"GET /test HTTP/1.1\r\n".to_vec());
        assert_eq!(
            result,
            Some(HttpRequest {
                method: "GET".to_string(),
                path: "/test".to_string(),
                body: Vec::new(),
            })
        );
    }

    #[test]
    fn parse_valid_post_request() {
        let result = Protocol::parse(&HttpProtocol, b"POST / HTTP/1.1\r\n".to_vec());
        assert_eq!(
            result,
            Some(HttpRequest {
                method: "POST".to_string(),
                path: "/".to_string(),
                body: Vec::new(),
            })
        );
    }

    #[test]
    fn parse_invalid_utf8_returns_none() {
        let invalid = vec![0xFF, 0xFE, 0x00];
        let result = Protocol::parse(&HttpProtocol, invalid);
        assert_eq!(result, None);
    }

    #[test]
    fn parse_missing_token_returns_none() {
        let result = Protocol::parse(&HttpProtocol, b"GET HTTP/1.1\r\n".to_vec());
        assert_eq!(result, None);
    }

    #[test]
    fn serialize_file_found_returns_200() {
        let response = HttpResponse::FileFound {
            content_type: "text/plain".to_string(),
            body: Vec::new(),
        };

        let result = Protocol::serialize(&HttpProtocol, response);
        assert_eq!(
            result,
            b"HTTP/1.1 200 OK\r\n\
            Content-Security-Policy: default-src 'self'; script-src 'self';\r\n\
            Content-Length: 0\r\n\
            Content-Type: text/plain\r\n\
            Connection: keep-alive\r\n\
            \r\n",
        );
    }

    #[test]
    fn serialize_file_not_found_returns_404() {
        let response = HttpResponse::NotFound;
        let result = Protocol::serialize(&HttpProtocol, response);
        assert_eq!(
            result,
            b"HTTP/1.1 404 Not Found\r\n\
            Content-Security-Policy: default-src 'self'; script-src 'self';\r\n\
            Content-Length: 22\r\n\
            Content-Type: text/plain\r\n\
            Connection: keep-alive\r\n\
            \r\n\
            Polaris\nFile Not Found",
        );
    }

    #[test]
    fn serialize_bad_request_returns_400() {
        let response = HttpResponse::BadRequest;
        let result = Protocol::serialize(&HttpProtocol, response);
        assert_eq!(
            result,
            b"HTTP/1.1 400 Bad Request\r\n\
            Content-Security-Policy: default-src 'self'; script-src 'self';\r\n\
            Content-Length: 19\r\n\
            Content-Type: text/plain\r\n\
            Connection: close\r\n\
            \r\n\
            Polaris\nBad Request",
        );
    }
}
