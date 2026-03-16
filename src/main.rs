use log::warn;
use polaris::{HttpProtocol, Router, Server};
use std::sync::Arc;
use std::fs;

static HTML_NOT_FOUND: &str = r#"<!DOCTYPE html>
    <html>
        <head><title>Polaris</title></head>
        <body>
            <h1>404 Not Found</h1>
            <h2>That URL doesn't exist</h2>
        </body>
    </html>
    "#;

#[tokio::main]
async fn main() {
    env_logger::init();

    let port = "127.0.0.1:8080";
    let protocol = HttpProtocol;

    let mut router = Router::new();
    router.add_route(b"/", home_html);
    router.add_route(b"/style.css", home_css);
    router.add_route(b"/script.js", home_js);
    router.add_route(b"/about", about_html);
    router.add_err_handler(handle_error);

    let server = Server::new(port, protocol, router)
        .await
        .expect("Failed to create server");

    let server = Arc::new(server);

    if let Err(e) = server.run().await {
        warn!("Failed to accept with error: {}", e);
    }
}

fn home_html(_: &[u8]) -> Vec<u8> {
    let bytes = fs::read("static/index.html");
    bytes.expect("No way ts didn't work")
}

fn home_css(_: &[u8]) -> Vec<u8> {
    let bytes = fs::read("static/style.css");
    bytes.expect("No way ts didn't work")
}

fn home_js(_: &[u8]) -> Vec<u8> {
    let bytes = fs::read("static/script.js");
    bytes.expect("No way ts didn't work")
}

fn about_html(_: &[u8]) -> Vec<u8> {
    let bytes = fs::read("static/about.html");
    bytes.expect("No way ts didn't work")
}

fn handle_error(_: &[u8]) -> Vec<u8> {
    HTML_NOT_FOUND.as_bytes().to_vec()
}
