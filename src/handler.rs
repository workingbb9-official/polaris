use crate::parser::{Message, Protocol};

static HTML_HOME: &str = r#"<!DOCTYPE html>
   <html>
        <head><title>Polaris</title></head>
        <body>
            <h1>Hello From Polaris<h1>
            <h2>A general purpose web-server<h2>
        </body>
    </html>
    "#;
static HTML_ABOUT: &str = r#"<!DOCTYPE html>
    <html>
        <head><title>Polaris</title></head>
        <body>
            <h1>About Polaris</h1>
            <h2>Name: Comes from the North Star</h2>
            <h2>Design: Async server with parsing and handling</h2>
        </body>
    </html>
    "#;
static HTML_NOT_FOUND: &str = r#"<!DOCTYPE html>
    <html>
        <head><title>Polaris</title></head>
        <body>
            <h1>404 Not Found</h1>
        </body>
    </html>
    "#;

pub fn handle_client(msg: &Message) -> String {
    match msg.protocol {
        Protocol::Http => handle_http(&msg.content),
        Protocol::Unknown => handle_unknown(),
    }
}

fn handle_http(req: &str) -> String {
    let (status, reply) = match req {
        "/" => ("200 OK", HTML_HOME),
        "/about" => ("200 OK", HTML_ABOUT),
        _ => ("404 NOT FOUND", HTML_NOT_FOUND),
    };
    let len = reply.len();

    format!(
        "HTTP/1.1 {status}\r\n\
        Content-Length: {len}\r\n\
        Content-Type: text/html\r\n\
        Connection: close\r\n\
        \r\n\
        {reply}"
    )
}

fn handle_unknown() -> String {
    "Unknown Protocol".to_string()
}
