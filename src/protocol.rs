pub trait Protocol {
    fn parse(&self, raw: &[u8]) -> Option<Vec<u8>>;
    fn format(&self, response: &HttpResponse) -> Vec<u8>;
}

pub struct HttpProtocol;

impl Protocol for HttpProtocol {
    fn parse(&self, raw: &[u8]) -> Option<Vec<u8>> {
        let request = match std::str::from_utf8(raw) {
            Ok(s) => s,
            Err(_) => return None,
        };

        let first_line = request.lines().next()?;
        let mut parts = first_line.split_whitespace();

        let _method = parts.next()?;
        let path = parts.next()?;
        let _version = parts.next()?;

        Some(path.as_bytes().to_vec())
    }

    fn format(&self, response: &HttpResponse) -> Vec<u8> {
        let header = format!(
            "HTTP/1.1 200 OK\r\n\
            Content-Security-Policy: default-src 'self'; script-src 'self';\r\n\
            Content-Length: {}\r\n\
            Content-Type: {}\r\n\
            Connection: keep-alive\r\n\
            \r\n",
            response.body.len(),
            response.content_type,
        );

        let mut final_response = header.into_bytes();

        final_response.extend(&response.body);
        final_response
    }
}

pub struct HttpResponse {
    body: Vec<u8>,
    content_type: String,
}

impl HttpResponse {
    pub fn new(body: Vec<u8>, content_type: String) -> Self {
        Self { body, content_type }
    }
}
