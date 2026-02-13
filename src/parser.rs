pub enum Protocol {
    Http,
    Unknown,
}

pub struct Message {
    pub protocol: Protocol,
    pub content: String,
}

pub fn parse_msg(msg: &str) -> Message {
    let first_line = msg.lines().next().unwrap_or("");
    let mut content: &str = "";

    let protocol = match first_line {
        s if s.starts_with("GET ") => {
            content = parse_http(first_line);
            Protocol::Http
        }
        s if s.starts_with("POST ") => {
            content = parse_http(first_line);
            Protocol::Http
        }
        s if s.starts_with("HTTP") => {
            content = parse_http(first_line);
            Protocol::Http
        }
        _ => Protocol::Unknown,
    };
    

    Message {
        protocol,
        content: content.to_string(),
    }
}

fn parse_http(first_line: &str) -> &str {
    let mut parts = first_line.split_whitespace();
    
    let _method = parts.next();
    let path = parts.next();
    let _proto = parts.next();

    path.unwrap_or("")
}
