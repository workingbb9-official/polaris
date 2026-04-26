/// A network address used to identify a remote device.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Addr {
    /// IPv4 address as raw bytes.
    pub octets: [u8; 4],
    /// Port number.
    pub port: u16,
}

/// An interface for I/O between devices.
pub trait Transport {
    type Error;

    fn broadcast_addr() -> Addr;
    fn send(&mut self, buf: &[u8], addr: Addr) -> Result<(), Self::Error>;
    fn recv(&mut self, buf: &mut [u8]) -> Result<(usize, Addr), Self::Error>;
}
