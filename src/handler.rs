use crate::parser::{Message, Protocol};

pub fn handle_client(msg: Message) -> String {
    match msg.protocol {
        Protocol::Http => handle_http(msg.content),
        Protocol::Unknown => handle_unknown(),
    }
}

fn handle_http(req: String) -> String {
    let reply = format!("Hello from Polaris\nYour request: {}", req);
    let response = format!(
        "HTTP/1.1 200 OK\r\n\
        Content-Length: {len}\r\n\
        Content-Type: text/plain\r\n\
        Connection: close\r\n\
        \r\n\
        {reply}",
        len = reply.len(),
        reply = reply
    );

    response
}

fn handle_unknown() -> String {
    "Unknown format".to_string()
}
