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

mod discovery;
mod heartbeat;

use discovery::DiscoveryAction;
use discovery::DiscoveryManager;

use heartbeat::HeartbeatAction;
use heartbeat::HeartbeatManager;

use crate::device::Device;
use crate::protocol::{DataMessage, DiscoveryMessage, HeartbeatMessage};

/// The significant events that can occur within a [Node].
#[derive(Debug, PartialEq, Eq)]
pub enum NodeEvent<'a> {
    /// The node has connected to a controller. This enables the sending and receiving of data from
    /// the controller.
    ControllerConnected,
    /// The node has received data from a controller. The buffer returned is a slice of the raw
    /// packet, removing the headers and extracting the data up to the length specified by the
    /// packet header.
    DataReceived { data: &'a [u8] },
}

#[derive(Debug)]
pub enum NodeAction<Addr> {
    /// Send out a [HeartbeatMessage] to the controller. This ensures that the controller remains
    /// connected to the node.
    SendHeartbeat { addr: Addr, msg: [u8; 3] },
    /// Send out a [DiscoveryMessage]. This should be sent over a broadcast address, so that any
    /// potential controllers can see it.
    SendDiscovery { msg: [u8; 3] },
}

/// The errors that can occur within a [Node].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeError {
    /// The node is not connected to a controller, which is necessary for the operation.
    NotConnected,
    /// The message sent was invalid. This could be due to an incorrect packet format that could
    /// not be parsed, or the message type was invalid for the current state of the node.
    InvalidMessage,
    /// The message received was sent from somebody that was not the connected controller. The
    /// message is dropped for security.
    WrongController,
}

/// A device which reports to a controller.
///
/// Nodes will remain simple and able to represent any type of device. They can be anything that
/// operates independently and sends information to a controller.
#[derive(Debug)]
pub struct Node<Addr> {
    dev: Device,
    controller: Option<Addr>,
    discovery: DiscoveryManager,
    heartbeat: HeartbeatManager,
}

impl<Addr: Copy + std::cmp::PartialEq> Node<Addr> {
    /// Create a new node.
    pub fn new(dev: Device, discovery_interval: u32, heartbeat_interval: u32) -> Self {
        Self {
            dev,
            controller: None,
            discovery: DiscoveryManager::new(discovery_interval),
            heartbeat: HeartbeatManager::new(heartbeat_interval),
        }
    }

    /// Extract the [Device] of the node.
    #[inline]
    pub fn dev(&self) -> Device {
        self.dev
    }

    /// Return an optional action to take based on current state.
    pub fn action(&mut self, now: u32) -> Option<NodeAction<Addr>> {
        if let Some(addr) = self.controller {
            if self.heartbeat.action(now) == HeartbeatAction::Send {
                let msg = HeartbeatMessage::new(self.dev.id());
                let raw = msg.to_bytes();
                Some(NodeAction::SendHeartbeat { addr, msg: raw })
            } else {
                None
            }
        } else if self.discovery.action(now) == DiscoveryAction::Broadcast {
            let msg = DiscoveryMessage::new_hello(self.dev.id());
            let raw = msg.to_bytes();
            Some(NodeAction::SendDiscovery { msg: raw })
        } else {
            None
        }
    }

    /// Process a packet and return event.
    pub fn process_msg<'a>(
        &mut self,
        raw: &'a [u8],
        addr: Addr,
    ) -> Result<Option<NodeEvent<'a>>, NodeError> {
        if let Some(con_addr) = self.controller.as_ref() {
            if addr != *con_addr {
                return Err(NodeError::WrongController);
            }

            DataMessage::from_bytes(raw).ok_or(NodeError::InvalidMessage)?;
            let len = raw[3];
            Ok(Some(NodeEvent::DataReceived {
                data: &raw[3..len as usize],
            }))
        } else {
            DiscoveryMessage::from_bytes(raw).ok_or(NodeError::InvalidMessage)?;
            self.controller = Some(addr);

            Ok(Some(NodeEvent::ControllerConnected))
        }
    }
}
