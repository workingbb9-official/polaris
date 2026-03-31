use log::warn;
use polaris::{HttpProtocol, HttpResponse, NetworkConfig, Server};
use std::fs;
use std::sync::Arc;
use std::time::Duration;

#[tokio::main]
async fn main() {
    env_logger::init();

    let port = "127.0.0.1:8080";

    let config = NetworkConfig::new(Duration::from_secs(5), 8192);

    let mut protocol = HttpProtocol::new();
    protocol.add_route("GET", "/", home_html);
    protocol.add_route("GET", "/style.css", home_css);
    protocol.add_route("GET", "/script.js", home_js);
    protocol.add_route("GET", "/about", about_html);
    protocol.add_route("GET", "/about.js", about_js);

    let server = Server::new(port, config, protocol)
        .await
        .expect("Failed to create server");

    let server = Arc::new(server);

    if let Err(e) = server.run().await {
        warn!("Failed to accept with error: {}", e);
    }
}

fn home_html(_: &[u8]) -> HttpResponse {
    let bytes = fs::read("examples/static/index.html").unwrap();
    HttpResponse::FileFound {
        content_type: "text/html".to_string(),
        body: bytes,
    }
}

fn home_css(_: &[u8]) -> HttpResponse {
    let bytes = fs::read("examples/static/style.css").unwrap();
    HttpResponse::FileFound {
        content_type: "text/css".to_string(),
        body: bytes,
    }
}

fn home_js(_: &[u8]) -> HttpResponse {
    let bytes = fs::read("examples/static/script.js").unwrap();
    HttpResponse::FileFound {
        content_type: "application/javascript".to_string(),
        body: bytes,
    }
}

fn about_html(_: &[u8]) -> HttpResponse {
    let bytes = fs::read("examples/static/about.html").unwrap();
    HttpResponse::FileFound {
        content_type: "text/html".to_string(),
        body: bytes,
    }
}

fn about_js(_: &[u8]) -> HttpResponse {
    let bytes = fs::read("examples/static/about.js").unwrap();
    HttpResponse::FileFound {
        content_type: "application/javascript".to_string(),
        body: bytes,
    }
}
