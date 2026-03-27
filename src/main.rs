use log::warn;
use polaris::{HttpProtocol, NetworkConfig, ProtocolResponse, Router, Server};
use std::sync::Arc;
use std::{fs, num::NonZero};

#[tokio::main]
async fn main() {
    env_logger::init();

    let port = "127.0.0.1:8080";

    let config = NetworkConfig::new(5, NonZero::new(1024).unwrap());

    let protocol = HttpProtocol;

    let mut router = Router::new();
    router.add_route(b"/", home_html);
    router.add_route(b"/style.css", home_css);
    router.add_route(b"/script.js", home_js);
    router.add_route(b"/about", about_html);
    router.add_route(b"/about.js", about_js);

    let server = Server::new(port, config, protocol, router)
        .await
        .expect("Failed to create server");

    let server = Arc::new(server);

    if let Err(e) = server.run().await {
        warn!("Failed to accept with error: {}", e);
    }
}

fn home_html(_: &[u8]) -> ProtocolResponse {
    let bytes = fs::read("static/index.html").unwrap();
    ProtocolResponse::FileFound {
        content_type: "text/html".to_string(),
        body: bytes,
    }
}

fn home_css(_: &[u8]) -> ProtocolResponse {
    let bytes = fs::read("static/style.css").unwrap();
    ProtocolResponse::FileFound {
        content_type: "text/css".to_string(),
        body: bytes,
    }
}

fn home_js(_: &[u8]) -> ProtocolResponse {
    let bytes = fs::read("static/script.js").unwrap();
    ProtocolResponse::FileFound {
        content_type: "application/javascript".to_string(),
        body: bytes,
    }
}

fn about_html(_: &[u8]) -> ProtocolResponse {
    let bytes = fs::read("static/about.html").unwrap();
    ProtocolResponse::FileFound {
        content_type: "text/html".to_string(),
        body: bytes,
    }
}

fn about_js(_: &[u8]) -> ProtocolResponse {
    let bytes = fs::read("static/about.js").unwrap();
    ProtocolResponse::FileFound {
        content_type: "application/javascript".to_string(),
        body: bytes,
    }
}
