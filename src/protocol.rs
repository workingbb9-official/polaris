pub trait Protocol {
    fn parse(&self, raw: &[u8]) -> Vec<u8>;
    fn format(&self, response: &[u8]) -> Vec<u8>;
}

pub struct HttpProtocol;

impl Protocol for HttpProtocol {
    fn parse(&self, raw: &[u8]) -> Vec<u8> {
        let request = match std::str::from_utf8(raw) {
            Ok(s) => s,
            Err(_) => return b"/error".to_vec(),
        };

        let first_line = match request.lines().next() {
            Some(line) => line,
            None => return b"/error".to_vec(),
        };

        let mut parts = first_line.split_whitespace();

        let _method = parts.next();

        let path = match parts.next() {
            Some(p) => p,
            None => return b"/error".to_vec(),
        };

        path.as_bytes().to_vec()
    }

    fn format(&self, response: &[u8]) -> Vec<u8> {
        let len = response.len();
        let res_to_str = std::str::from_utf8(response).unwrap_or("404 Response Not Found");

        format!(
            "HTTP/1.1 200 OK\r\n\
Content-Security-Policy: default-src 'self'; script-src 'self';
Content-Length: {len}\r\n\
Content-Type: text/html\r\n\
Connection: keep-alive\r\n\
\r\n\
{res_to_str}"
        )
        .into_bytes()
    }
}
