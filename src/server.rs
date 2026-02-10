use std::net::{TcpListener, TcpStream};
use std::io::{self, Read, Write};

pub enum ServerState {
    Listening,
    Processing,
}

pub struct NetworkServer {
    listener: TcpListener,
    state: ServerState,
    client: Option<TcpStream>,
}

impl NetworkServer {
    pub fn new(addr: &str) -> io::Result<Self> {
        let listener = TcpListener::bind(addr)?;
        listener.set_nonblocking(true)?;

        Ok(Self {
            listener,
            state: ServerState::Listening,
            client: None,
        })
    }

    pub fn run(&mut self) {
        loop {
            match self.state {
                ServerState::Listening => self.handle_listening(),
                ServerState::Processing => self.handle_processing(),
            }
        }
    }

    fn handle_listening(&mut self) {
        let Ok((stream, addr)) = self.listener.accept() else {
            return;
        };

        println!("Connected to {}", addr);

        stream.set_nonblocking(true).expect("Failed to set non-blocking");
        self.client = Some(stream);
        self.state = ServerState::Processing;
    }

    fn handle_processing(&mut self) {
        let Some(ref mut stream) = self.client else {
            self.state = ServerState::Listening;
            return;
        };

        let mut buffer = [0u8; 1024];

        match stream.read(&mut buffer) {
            Ok(0) => {
                println!("Client closed connection");
                self.disconnect();
            }
            Ok(n) => {
                println!("Read {} bytes {:?}", n, &buffer[..n]);
                let response = "HTTP/1.1 200 OK\r\n\
                                Content-Type: text/plain\r\n\
                                Content-Length: 12\r\n\r\n\
                                Hello World!";
                let _ = stream.write_all(response.as_bytes());
            }
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                // Keep searching for data
            }
            Err(_) => {
                println!("Communication error");
                self.disconnect();
            }
        }
    }

    fn disconnect(&mut self) {
        self.client = None;
        self.state = ServerState::Listening;
    }
}
