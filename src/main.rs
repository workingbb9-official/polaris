mod server;
use server::NetworkServer;
use std::io;

fn main() -> io::Result<()> {
    env_logger::init();
    let mut s1 = NetworkServer::new("127.0.0.1:8080")?;
    let mut s2 = NetworkServer::new("127.0.0.1:9000")?;

    loop {
        s1.run();
        s2.run();
    }
}
