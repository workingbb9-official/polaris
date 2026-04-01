mod http;
pub use http::HttpMessage;
pub use http::HttpProtocol;
pub use http::HttpResponse;
pub use http::{Connection, ContentType, Status};

#[trait_variant::make(Protocol: Send)]
pub trait _P {
    type Message;
    type Response;

    fn framing(&self) -> Framing;
    fn parse(&self, raw: Vec<u8>) -> Option<Self::Message>;
    fn route(&self, msg: Self::Message) -> Self::Response;
    fn serialize(&self, response: Self::Response) -> Vec<u8>;
}

pub enum Framing {
    Delimiter(&'static [u8]),
    ExactBytes(usize),
    Http,
}
