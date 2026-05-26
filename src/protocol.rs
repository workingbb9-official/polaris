// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 workingbb9-official

#![allow(dead_code)]

use crate::device::{Device, DeviceId};
use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub(crate) enum MessageType {
    Unknown = 0x00,
    Hello = 0x01,
    Welcome = 0x02,
    Data = 0x03,
    Heartbeat = 0x04,
}

impl TryFrom<u8> for MessageType {
    type Error = ();
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x01 => Ok(MessageType::Hello),
            0x02 => Ok(MessageType::Welcome),
            0x03 => Ok(MessageType::Data),
            0x04 => Ok(MessageType::Heartbeat),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, Copy, FromBytes, IntoBytes, Immutable, KnownLayout)]
#[repr(C, packed)]
pub(crate) struct Packet<T> {
    pub msg_type: u8,
    pub payload: T,
}

impl<P> Packet<P> {
    pub fn new(msg_type: MessageType, payload: P) -> Self {
        Self {
            msg_type: msg_type as u8,
            payload,
        }
    }
}

#[derive(Debug, Clone, Copy, FromBytes, IntoBytes, Immutable, KnownLayout)]
#[repr(C, packed)]
pub(crate) struct HelloMessage {
    pub dev: Device,
    pub heartbeat_interval: u32,
}

#[derive(Debug, Clone, Copy, FromBytes, IntoBytes, Immutable, KnownLayout)]
#[repr(C, packed)]
pub(crate) struct WelcomeMessage {
    pub dev: Device,
    pub heartbeat_interval: u32,
}

#[derive(Debug, Clone, FromBytes, IntoBytes, Immutable, KnownLayout)]
#[repr(C, packed)]
pub(crate) struct DataMessage {
    pub from: DeviceId,
    pub len: u8,
    pub payload: [u8; 255],
}

#[derive(Debug, Clone, Copy, FromBytes, IntoBytes, Immutable, KnownLayout)]
#[repr(C, packed)]
pub(crate) struct HeartbeatMessage {
    pub from: DeviceId,
}
