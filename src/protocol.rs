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

use crate::device::DeviceId;

pub(crate) const MSG_TYPE_HELLO: u8 = 0x01;
pub(crate) const MSG_TYPE_WELCOME: u8 = 0x02;
pub(crate) const MSG_TYPE_DATA: u8 = 0x04;

#[derive(Debug, Clone)]
pub(crate) struct DataMessage {
    from: DeviceId,
    payload: [u8; 256],
    len: usize,
}

impl DataMessage {
    pub(crate) fn new(from: DeviceId, payload: &[u8]) -> Self {
        let mut buf = [0u8; 256];
        let len = payload.len().min(256);
        buf[..len].copy_from_slice(&payload[..len]);

        Self {
            from,
            payload: buf,
            len,
        }
    }

    #[inline]
    pub(crate) fn from(&self) -> DeviceId {
        self.from
    }

    #[inline]
    pub(crate) fn payload(&self) -> &[u8] {
        &self.payload[..self.len]
    }

    pub(crate) fn to_bytes(&self) -> [u8; 263] {
        let mut buf = [0u8; 263];
        buf[0] = MSG_TYPE_DATA;
        buf[1..3].copy_from_slice(&self.from.value().to_be_bytes());
        buf[3..5].copy_from_slice(&(self.len as u16).to_be_bytes());
        buf[5..self.len + 5].copy_from_slice(self.payload());

        buf
    }
}

#[derive(Debug, Clone)]
pub(crate) enum DiscoveryMessage {
    Hello(DeviceId),
    Welcome(DeviceId),
}

impl DiscoveryMessage {
    pub(crate) fn new_hello(node_id: DeviceId) -> Self {
        DiscoveryMessage::Hello(node_id)
    }

    pub(crate) fn new_welcome(controller_id: DeviceId) -> Self {
        DiscoveryMessage::Welcome(controller_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_message_oversized_payload() {
        let buf = [0xFFu8; 300];
        let msg = DataMessage::new(DeviceId::new(7), &buf);

        assert_eq!(msg.len, 256);
        assert_eq!(msg.payload().len(), 256);
    }

    #[test]
    fn test_new_message_undersized_payload() {
        let buf = [0xFFu8; 128];
        let msg = DataMessage::new(DeviceId::new(7), &buf);

        assert_eq!(msg.len, 128);
        assert_eq!(msg.payload().len(), 128);
    }
}
