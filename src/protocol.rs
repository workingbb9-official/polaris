pub trait Protocol {
    fn parse_req(&self, raw: Vec<u8>) -> Option<ProtocolRequest>;
    fn format_resp(&self, response: ProtocolResponse) -> Vec<u8>;
}

pub enum ProtocolRequest {
    Http(HttpRequest),
    Raw(Vec<u8>),
}

pub enum ProtocolResponse {
    Http(HttpResponse),
    Raw(Vec<u8>),
}

impl ProtocolResponse {
    pub fn new_http(status: String, body: Vec<u8>, content_type: String) -> Self {
        Self::Http(HttpResponse {
            status,
            body,
            content_type,
        })
    }
}

pub struct HttpRequest {
    method: String,
    path: String,
    body: Vec<u8>,
}

impl HttpRequest {
    pub fn new(method: String, path: String, body: Vec<u8>) -> Self {
        Self { method, path, body }
    }

    pub fn path(&self) -> &str {
        &self.path
    }
}

pub struct HttpResponse {
    status: String,
    body: Vec<u8>,
    content_type: String,
}

impl HttpResponse {
    pub fn new(status: String, body: Vec<u8>, content_type: String) -> Self {
        Self {
            status,
            body,
            content_type,
        }
    }
}

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

        let http_req = ProtocolRequest::Http(HttpRequest {
            method: method.to_string(),
            path: path.to_string(),
            body: Vec::new(),
        });

        Some(http_req)
    }

    /// Create a binary response header from HttpResponse.
    fn format_resp(&self, response: ProtocolResponse) -> Vec<u8> {
        let http_res = match response {
            ProtocolResponse::Http(res) => res,
            _ => return Vec::new(),
        };

        let header = format!(
            "{}\r\n\
            Content-Security-Policy: default-src 'self'; script-src 'self';\r\n\
            Content-Length: {}\r\n\
            Content-Type: {}\r\n\
            Connection: keep-alive\r\n\
            \r\n",
            http_res.status,
            http_res.body.len(),
            http_res.content_type,
        );

        let mut final_response = header.into_bytes();

        // Add body after header
        final_response.extend(&http_res.body);

        final_response
    }
}
