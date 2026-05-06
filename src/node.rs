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

/// The core interface for [Node] logic.
pub trait NodeApp {
    /// Called when a [DataMessage] is received.
    fn on_data(&mut self, data: &[u8]);
    /// Called when a welcome message is received during discovery phase, and the node is not
    /// already connected to a controller.
    fn on_connection(&mut self);
}

/// The significant events that can occur within a [Node].
#[derive(Debug, PartialEq, Eq)]
pub enum NodeEvent {
    /// The node has connected to a controller. This enables the sending and receiving of data from
    /// the controller.
    ControllerConnected,
    /// The controller has sent a [DataMessage]. The node will pass the data onto the injected
    /// handler. This event is a notification, the data has already been addressed.
    DataReceived,
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
pub struct Node<Addr, App: NodeApp> {
    dev: Device,
    controller: Option<Addr>,
    discovery: DiscoveryManager,
    heartbeat: HeartbeatManager,
    app: App,
}

impl<Addr: Copy + std::cmp::PartialEq, App: NodeApp> Node<Addr, App> {
    /// Create a new node.
    pub fn new(dev: Device, discovery_interval: u32, heartbeat_interval: u32, app: App) -> Self {
        Self {
            dev,
            controller: None,
            discovery: DiscoveryManager::new(discovery_interval),
            heartbeat: HeartbeatManager::new(heartbeat_interval),
            app,
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
    ///
    /// # Errors
    /// * `Err(NodeError::WrongController)` - Received a packet from a different address.
    /// * `Err(NodeError::InvalidMessage)` - The message received was invalid. This could be due to
    ///   an incorrect format, or a wrong packet type for the current Node state.
    pub fn process_msg(&mut self, buf: &[u8], addr: Addr) -> Result<Option<NodeEvent>, NodeError> {
        if let Some(con_addr) = self.controller.as_ref() {
            if addr != *con_addr {
                return Err(NodeError::WrongController);
            }

            if let Some(msg) = DataMessage::from_bytes(buf) {
                self.app.on_data(msg.payload());
            } else {
                return Err(NodeError::InvalidMessage);
            }

            Ok(Some(NodeEvent::DataReceived))
        } else {
            if let Some(DiscoveryMessage::Welcome) = DiscoveryMessage::from_bytes(buf) {
                self.controller = Some(addr);
                self.app.on_connection();
            } else {
                return Err(NodeError::InvalidMessage);
            }

            Ok(Some(NodeEvent::ControllerConnected))
        }
    }
}
