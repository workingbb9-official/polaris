mod server;
use server::NetworkServer;
use std::io;

fn main() -> io::Result<()> {
    let mut server = NetworkServer::new("127.0.0.1:8080")?;
    server.run();

    Ok(())
}
