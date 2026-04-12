// Copyright (c) 2026 workingbb9-official
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#![allow(dead_code)]

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

#[derive(Debug, Clone)]
pub(crate) struct Message {
    from: DeviceId,
    to: DeviceId,
    payload: [u8; 256],
    len: usize,
}

impl Message {
    pub(crate) fn new(from: DeviceId, to: DeviceId, payload: &[u8]) -> Self {
        let mut buf = [0u8; 256];
        let len = payload.len().min(256);
        buf[..len].copy_from_slice(&payload[..len]);

        Self {
            from,
            to,
            payload: buf,
            len,
        }
    }

    #[inline]
    pub(crate) fn from(&self) -> DeviceId {
        self.from
    }

    #[inline]
    pub(crate) fn to(&self) -> DeviceId {
        self.to
    }

    #[inline]
    pub(crate) fn payload(&self) -> &[u8] {
        &self.payload[..self.len]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_message_oversized_payload() {
        let buf = [0xFFu8; 300];
        let msg = Message::new(DeviceId::new(7), DeviceId::new(10), &buf);

        assert_eq!(msg.len, 256);
        assert_eq!(msg.payload().len(), 256);
    }

    #[test]
    fn test_new_message_undersized_payload() {
        let buf = [0xFFu8; 128];
        let msg = Message::new(DeviceId::new(7), DeviceId::new(10), &buf);

        assert_eq!(msg.len, 128);
        assert_eq!(msg.payload().len(), 128);
    }
}
