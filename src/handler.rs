pub async fn handle_client(msg: &str) -> String {
    let has_get = msg.starts_with("GET");
    let has_http = msg.starts_with("HTTP");

    if has_get || has_http {
        return handle_http().await;
    }

    "Unknown".to_string()
}

async fn handle_http() -> String {
    let content = "Hello from Polaris";
    let response = format!(
        "HTTP/1.1 200 OK\r\n\
        Content-Length: {len}\r\n\
        Content-Type: text/plain\r\n\
        Connection: close\r\n\
        \r\n\
        {content}",
        len = content.len(),
        content = content
    );

    response
}
