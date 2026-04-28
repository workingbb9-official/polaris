use std::fmt::Debug;

/// An interface for I/O between devices.
pub trait Transport {
    type Addr: PartialEq + Debug;
    type Error;

    fn broadcast_addr(&mut self) -> Self::Addr;
    fn send(&mut self, buf: &[u8], addr: &Self::Addr) -> Result<(), Self::Error>;
    fn recv(&mut self, buf: &mut [u8]) -> Result<(usize, Self::Addr), Self::Error>;
}
