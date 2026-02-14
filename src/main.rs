use log::warn;
use polaris::{HttpProtocol, Router, Server};
use std::sync::Arc;

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

static HTML_HOME: &str = r#"<!DOCTYPE html>
   <html>
        <head><title>Polaris</title></head>
        <body>
            <h1>Hello From Polaris<h1>
            <h2>A general purpose web-server<h2>
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

#[tokio::main]
async fn main() {
    env_logger::init();

    let port = "127.0.0.1:8080";
    let protocol = HttpProtocol;

    let mut router = Router::new();
    router.add_route(b"/", handle_home);
    router.add_route(b"/about", handle_about);
    router.add_route(b"/error", handle_error);

    let server = Server::new(port, protocol, router)
        .await
        .expect("Failed to create server");

    let server = Arc::new(server);

    if let Err(e) = server.run().await {
        warn!("Failed to accept with error: {}", e);
    }
}

fn handle_home(_: &[u8]) -> Vec<u8> {
    HTML_HOME.as_bytes().to_vec()
}

fn handle_about(_: &[u8]) -> Vec<u8> {
    HTML_ABOUT.as_bytes().to_vec()
}

fn handle_error(_: &[u8]) -> Vec<u8> {
    HTML_NOT_FOUND.as_bytes().to_vec()
}
