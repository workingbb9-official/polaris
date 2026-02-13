pub enum Protocol {
    Http,
    Unknown,
}

pub struct Message {
    pub protocol: Protocol,
    pub content: String,
}

pub fn parse_msg(msg: &str) -> Message {
    let content = msg.lines().next().unwrap_or("");
    let protocol = match content {
        s if s.starts_with("GET ") => Protocol::Http,
        s if s.starts_with("POST ") => Protocol::Http,
        s if s.starts_with("HTTP") => Protocol::Http,
        _ => Protocol::Unknown,
    };

    Message {
        protocol,
        content: content.to_string(),
    }
}
