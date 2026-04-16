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

use std::future::Future;
use std::net::SocketAddr;

use crate::device::DeviceId;

pub(crate) const MSG_TYPE_HELLO: u8 = 0x01;
pub(crate) const MSG_TYPE_WELCOME: u8 = 0x02;
pub(crate) const MSG_TYPE_DATA: u8 = 0x03;
pub(crate) const MSG_TYPE_HEARTBEAT: u8 = 0x04;

pub trait Transport {
    type Error;

    fn send(
        &mut self,
        buf: &[u8],
        addr: SocketAddr,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send;
    fn recv(
        &mut self,
        buf: &mut [u8],
    ) -> impl Future<Output = Result<(usize, SocketAddr), Self::Error>> + Send;
}

pub(crate) enum MessageType {
    Hello,
    Welcome,
    Data,
    Heartbeat,
}

impl MessageType {
    pub(crate) fn from_buf(buf: &[u8]) -> Option<Self> {
        match buf[0] {
            MSG_TYPE_HELLO => Some(MessageType::Hello),
            MSG_TYPE_WELCOME => Some(MessageType::Welcome),
            MSG_TYPE_DATA => Some(MessageType::Data),
            MSG_TYPE_HEARTBEAT => Some(MessageType::Heartbeat),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct DataMessage {
    from: DeviceId,
    payload: [u8; 255],
    len: usize,
}

impl DataMessage {
    pub(crate) fn new(from: DeviceId, payload: &[u8]) -> Self {
        let mut buf = [0u8; 255];
        let len = payload.len().min(255);
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

    pub(crate) fn to_bytes(&self) -> [u8; 259] {
        let mut buf = [0u8; 259];
        buf[0] = MSG_TYPE_DATA;
        buf[1..3].copy_from_slice(&self.from.value().to_be_bytes());
        buf[3] = self.len as u8;
        buf[4..self.len + 4].copy_from_slice(self.payload());

        buf
    }

    pub(crate) fn from_bytes(buf: &[u8]) -> Option<Self> {
        if buf.len() < 4 {
            return None;
        }

        let from = DeviceId::new(u16::from_be_bytes([buf[1], buf[2]]));
        let len = buf[3] as usize;

        if buf.len() < len + 4 {
            return None;
        }

        let mut payload = [0u8; 255];
        payload[..len].copy_from_slice(&buf[4..len + 4]);

        Some(Self { from, payload, len })
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

    pub(crate) fn to_bytes(&self) -> [u8; 3] {
        let mut buf = [0u8; 3];

        match self {
            DiscoveryMessage::Hello(node_id) => {
                buf[0] = MSG_TYPE_HELLO;
                buf[1..3].copy_from_slice(&node_id.value().to_be_bytes());
                buf
            }
            DiscoveryMessage::Welcome(controller_id) => {
                buf[0] = MSG_TYPE_WELCOME;
                buf[1..3].copy_from_slice(&controller_id.value().to_be_bytes());
                buf
            }
        }
    }

    pub(crate) fn from_bytes(buf: &[u8]) -> Option<Self> {
        if buf.len() < 3 {
            return None;
        }

        match buf[0] {
            MSG_TYPE_HELLO => {
                let node_id = DeviceId::new(u16::from_be_bytes([buf[1], buf[2]]));
                Some(Self::Hello(node_id))
            }
            MSG_TYPE_WELCOME => {
                let controller_id = DeviceId::new(u16::from_be_bytes([buf[1], buf[2]]));
                Some(Self::Welcome(controller_id))
            }
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct HeartbeatMessage {
    from: DeviceId,
}

impl HeartbeatMessage {
    pub(crate) fn new(from: DeviceId) -> Self {
        Self { from }
    }

    pub(crate) fn to_bytes(&self) -> [u8; 3] {
        let mut buf = [0u8; 3];

        buf[0] = MSG_TYPE_HEARTBEAT;
        buf[1..3].copy_from_slice(&self.from.value().to_be_bytes());
        buf
    }

    pub(crate) fn from_bytes(buf: &[u8]) -> Option<Self> {
        if buf.len() < 3 {
            return None;
        }

        let from = DeviceId::new(u16::from_be_bytes([buf[1], buf[2]]));
        Some(Self { from })
    }

    pub(crate) fn from(&self) -> DeviceId {
        self.from
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_message_oversized_payload() {
        let buf = [0xFFu8; 300];
        let msg = DataMessage::new(DeviceId::new(7), &buf);

        assert_eq!(msg.len, 255);
        assert_eq!(msg.payload().len(), 255);
    }

    #[test]
    fn test_new_message_undersized_payload() {
        let buf = [0xFFu8; 128];
        let msg = DataMessage::new(DeviceId::new(7), &buf);

        assert_eq!(msg.len, 128);
        assert_eq!(msg.payload().len(), 128);
    }

    #[test]
    fn test_data_message_trip() {
        let from = DeviceId::new(10);
        let payload = [0xABu8; 128];

        let msg = DataMessage::new(from, &payload);
        let bytes = msg.to_bytes();
        let parsed = DataMessage::from_bytes(&bytes).unwrap();

        assert_eq!(parsed.from(), from);
        assert_eq!(parsed.payload(), msg.payload());
    }

    #[test]
    fn test_data_message_too_short() {
        assert!(DataMessage::from_bytes(&[0x04, 0x00]).is_none());
    }

    #[test]
    fn test_data_message_long_len() {
        let mut buf = [0u8; 128];
        let payload = [0xABu8; 64];

        buf[0] = MSG_TYPE_DATA;
        buf[1..3].copy_from_slice(&10_u16.to_be_bytes());
        buf[3] = 255;
        buf[4..68].copy_from_slice(&payload);

        assert!(DataMessage::from_bytes(&buf).is_none());
    }

    #[test]
    fn test_discovery_message_trip() {
        let node_id = DeviceId::new(10);

        let msg = DiscoveryMessage::new_hello(node_id);
        let bytes = msg.to_bytes();
        let parsed = DiscoveryMessage::from_bytes(&bytes);

        assert!(matches!(parsed, Some(DiscoveryMessage::Hello(id)) if id == node_id));
    }

    #[test]
    fn test_heartbeat_message_trip() {
        let id = DeviceId::new(10);

        let msg = HeartbeatMessage::new(id);
        let bytes = msg.to_bytes();
        let parsed = HeartbeatMessage::from_bytes(&bytes).unwrap();

        assert_eq!(parsed.from(), id);
    }
}
