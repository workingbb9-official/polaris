mod http;
pub use http::HttpProtocol;

pub trait Protocol {
    fn parse_req(&self, raw: Vec<u8>) -> Option<ProtocolRequest>;
    fn serialize_resp(&self, response: ProtocolResponse) -> Vec<u8>;
}

pub enum ProtocolRequest {
    Http {
        method: String,
        path: String,
        body: Vec<u8>,
    },
    Raw(Vec<u8>),
}

pub enum ProtocolResponse {
    FileFound { content_type: String, body: Vec<u8> },
    FileNotFound,
    BadRequest,
}
