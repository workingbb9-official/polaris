use super::*;
use std::collections::HashMap;

type HttpHandler = fn(&[u8]) -> HttpResponse;

#[derive(PartialEq, Debug)]
pub struct HttpMessage {
    method: String,
    path: String,
    body: Vec<u8>,
}

pub struct HttpResponse {
    pub status: Status,
    pub connection: Connection,
    pub body: Option<(ContentType, Vec<u8>)>,
}

pub enum Status {
    OK,
    NoContent,
    NotFound,
    BadRequest,
}

impl Status {
    fn as_str(&self) -> &'static str {
        match self {
            Status::OK => "200 OK",
            Status::NoContent => "204 No Content",
            Status::NotFound => "404 Not Found",
            Status::BadRequest => "400 Bad Request",
        }
    }
}

pub enum Connection {
    KeepAlive,
    Close,
}

impl Connection {
    fn as_str(&self) -> &'static str {
        match self {
            Connection::KeepAlive => "keep-alive",
            Connection::Close => "close",
        }
    }
}

pub enum ContentType {
    Plain,
    Html,
    Css,
    JavaScript,
}

impl ContentType {
    fn as_str(&self) -> &'static str {
        match self {
            ContentType::Plain => "text/plain",
            ContentType::Html => "text/html",
            ContentType::Css => "text/css",
            ContentType::JavaScript => "text/javascript",
        }
    }
}

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
        Framing::Http
    }

    fn parse(&self, raw: Vec<u8>) -> Option<HttpMessage> {
        let request = String::from_utf8(raw).ok()?;
        let mut parts = request.splitn(2, "\r\n\r\n");
        let headers = parts.next()?;

        let value = url_decode(parts.next().unwrap_or(""));
        let body_str = value.split_once('=').map(|x| x.1).unwrap_or(&value);
        let body = body_str.as_bytes().to_vec();

        let first_line = headers.lines().next()?;
        let mut tokens = first_line.split_whitespace();

        let method = tokens.next()?;
        let path = tokens.next()?;
        let _version = tokens.next()?;

        let http_req = HttpMessage {
            method: method.to_string(),
            path: path.to_string(),
            body,
        };

        Some(http_req)
    }

    fn route(&self, msg: HttpMessage) -> HttpResponse {
        let key = format!("{} {}", msg.method, msg.path);

        if let Some(handler) = self.routes.get(&key) {
            return handler(&msg.body[..]);
        }

        HttpResponse {
            status: Status::NotFound,
            connection: Connection::KeepAlive,
            body: Some((ContentType::Plain, b"Polaris\nNotFound".to_vec())),
        }
    }

    fn serialize(&self, response: HttpResponse) -> Vec<u8> {
        let status_str = response.status.as_str();
        let conn_str = response.connection.as_str();

        let (content_str, body) = match response.body {
            Some((ct, body)) => (ct.as_str(), body),
            None => ("", Vec::new()),
        };

        build_response(status_str, conn_str, content_str, body)
    }
}

fn build_response(status: &str, conn: &str, content_type: &str, body: Vec<u8>) -> Vec<u8> {
    let header = format!(
        "HTTP/1.1 {}\r\n\
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

fn url_decode(input: &str) -> String {
    let mut result = String::new();
    let mut chars = input.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '+' => result.push(' '),
            '%' => {
                let h1 = chars.next().unwrap_or('0');
                let h2 = chars.next().unwrap_or('0');
                if let Ok(byte) = u8::from_str_radix(&format!("{h1}{h2}"), 16) {
                    result.push(byte as char);
                }
            }
            _ => result.push(c),
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_valid_get_request() {
        let protocol = HttpProtocol::new();
        let result = protocol.parse(b"GET /test HTTP/1.1\r\n".to_vec());

        assert_eq!(
            result,
            Some(HttpMessage {
                method: "GET".to_string(),
                path: "/test".to_string(),
                body: Vec::new(),
            })
        );
    }

    #[test]
    fn parse_valid_post_request() {
        let protocol = HttpProtocol::new();
        let result = protocol.parse(b"POST / HTTP/1.1\r\n".to_vec());

        assert_eq!(
            result,
            Some(HttpMessage {
                method: "POST".to_string(),
                path: "/".to_string(),
                body: Vec::new(),
            })
        );
    }

    #[test]
    fn parse_invalid_utf8_returns_none() {
        let invalid = vec![0xFF, 0xFE, 0x00];

        let protocol = HttpProtocol::new();
        let result = protocol.parse(invalid);

        assert_eq!(result, None);
    }

    #[test]
    fn parse_missing_token_returns_none() {
        let protocol = HttpProtocol::new();
        let result = protocol.parse(b"GET HTTP/1.1\r\n".to_vec());

        assert_eq!(result, None);
    }
}
