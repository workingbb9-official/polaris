/// A unique identifier for a Device.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DeviceId(u64);

impl DeviceId {
    /// Creates a new DeviceId.
    #[inline]
    pub fn new(val: u64) -> Self {
        Self(val)
    }

    /// Accesses the numeric value of the ID.
    #[inline]
    pub fn value(&self) -> u64 {
        self.0
    }
}
