use log::warn;
use polaris::{HttpProtocol, HttpResponse, Router, Server};
use std::fs;
use std::sync::Arc;

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
    router.add_route(b"/about.js", about_js);
    router.add_err_handler(handle_error);

    let server = Server::new(port, protocol, router)
        .await
        .expect("Failed to create server");

    let server = Arc::new(server);

    if let Err(e) = server.run().await {
        warn!("Failed to accept with error: {}", e);
    }
}

fn home_html(_: &[u8]) -> HttpResponse {
    let bytes = fs::read("static/index.html").unwrap();
    HttpResponse::new(bytes, "text/html".to_string())
}

fn home_css(_: &[u8]) -> HttpResponse {
    let bytes = fs::read("static/style.css").unwrap();
    HttpResponse::new(bytes, "text/css".to_string())
}

fn home_js(_: &[u8]) -> HttpResponse {
    let bytes = fs::read("static/script.js").unwrap();
    HttpResponse::new(bytes, "application/javascript".to_string())
}

fn about_html(_: &[u8]) -> HttpResponse {
    let bytes = fs::read("static/about.html").unwrap();
    HttpResponse::new(bytes, "text/html".to_string())
}

fn about_js(_: &[u8]) -> HttpResponse {
    let bytes = fs::read("static/about.js").unwrap();
    HttpResponse::new(bytes, "text/html".to_string())
}

fn handle_error(_: &[u8]) -> HttpResponse {
    HttpResponse::new(HTML_NOT_FOUND.as_bytes().to_vec(), "text/html".to_string())
}
