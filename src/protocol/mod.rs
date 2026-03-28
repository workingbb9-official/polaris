mod http;
pub use http::HttpProtocol;

#[trait_variant::make(Protocol: Send)]
pub trait _P {
    fn framing(&self) -> Framing;
    fn parse(&self, raw: Vec<u8>) -> Option<ProtocolRequest>;
    fn serialize(&self, response: ProtocolResponse) -> Vec<u8>;
}

pub enum ProtocolRequest {
    Http {
        method: String,
        path: String,
        body: Vec<u8>,
    },
    Raw(Vec<u8>),
}

#[derive(Debug, PartialEq)]
pub enum ProtocolResponse {
    FileFound { content_type: String, body: Vec<u8> },
    FileNotFound,
    BadRequest,
}

pub enum Framing {
    Delimiter(&'static [u8]),
    ExactBytes(usize),
}
