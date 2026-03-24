mod http;
use crate::network::Network;
pub use http::HttpProtocol;
use log::info;

#[trait_variant::make(Protocol: Send)]
pub trait _P {
    async fn read(&self, network: &mut Network) -> Option<Vec<u8>>;

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

pub enum ProtocolResponse {
    FileFound { content_type: String, body: Vec<u8> },
    FileNotFound,
    BadRequest,
}
