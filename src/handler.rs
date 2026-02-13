use crate::parser::{Message, Protocol};

pub fn handle_client(msg: Message) -> String {
    match msg.protocol {
        Protocol::Http => handle_http(&msg.content),
        Protocol::Unknown => handle_unknown(),
    }
}

fn handle_http(req: &str) -> String {
    let page = match req {
        "/" =>
            Some(format!(
                r#"
                <!DOCTYPE html>
                <html>
                    <head><title>Polaris</title></head>
                    <body>
                        <h1>Hello from Polaris<h1>
                        <h2>You said: {req}<h2>
                    </body>
                </html>
                "#,
                req = req
            )),
        "/about" =>
            Some(format!(
                r#"
                <!DOCTYPE html>
                <html>
                    <head><title>Polaris</title></head>
                    <body>
                        <h1>About Polaris<h1>
                        <h2>A general purpose web-server<h2>
                    </body>
                </html>
                "#,
            )),
        _ => None,
    };
        
    let response = match page {
        Some(s) =>
            format!(
                "HTTP/1.1 200 OK\r\n\
                Content-Length: {len}\r\n\
                Content-Type: text/html\r\n\
                Connection: close\r\n\
                \r\n\
                {reply}",
                len = s.len(),
                reply = s
            ),
        None =>
            "HTTP/1.1 404 Not Found\r\n
            Content-Length: 13\r\n
            Content-Type: text/plain\r\n
            Connection: close\r\n
            \r\n
            404 Not Found".to_string(),
    };

    response
}

fn handle_unknown() -> String {
    "Unknown format".to_string()
}
