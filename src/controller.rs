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

use std::collections::HashMap;
use std::time::Instant;

use crate::device::{Device, DeviceId};
use crate::protocol::{DataMessage, DiscoveryMessage, HeartbeatMessage, MessageType};
use crate::{Addr, Transport};

/// The errors that can occur within a [Controller]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControllerError<T: Transport> {
    /// Used on discovery to represent that the DeviceId given is already in use by a stored node or
    /// the controller itself.
    DeviceIdInUse,
    /// Error sending or receiving, holds the specific error from the Transport trait.
    TransportError(T::Error),
    /// The message received was invalid. This could be returned if the bytes could not be parsed
    /// into a 'Message' or the message type was incorrect for the current state.
    InvalidMessage,
    /// The [DeviceId] received was not found within the Controller registry.
    DeviceNotRegistered,
}

/// The main orchestrator of the system.
///
/// All devices communicate through a Controller. Main logic will be decided here, allowing the
/// nodes to stay simple and do their specific job. Because it is more complex, it is recommended
/// for a Controller to be a device that has more resources in order to stay responsive while
/// maintaining the coordination of the nodes.
pub struct Controller<T: Transport> {
    dev: Device,
    nodes: HashMap<DeviceId, (Addr, Instant)>,
    max_nodes: usize,
    transport: T,
}

impl<T: Transport> Controller<T> {
    /// Create a new Controller.
    pub fn new(dev: Device, max_nodes: usize, transport: T) -> Self {
        Self {
            dev,
            nodes: HashMap::new(),
            max_nodes,
            transport,
        }
    }

    /// Extract the [Device] of the controller.
    pub fn dev(&self) -> Device {
        self.dev
    }

    /// Access the last time a node was seen.
    ///
    /// # Errors
    /// * Err(ControllerError::DeviceNotRegistered) - The [DeviceId] was not found in the registry.
    pub fn last_seen(&self, id: DeviceId) -> Result<Instant, ControllerError<T>> {
        match self.nodes.get(&id) {
            Some((_, last_seen)) => Ok(*last_seen),
            None => Err(ControllerError::DeviceNotRegistered),
        }
    }

    /// Add a node to the registry.
    ///
    /// # Errors
    /// * Err(ControllerError::DeviceIdInUse) - DeviceId being added is already in the registry.
    pub fn add_node(&mut self, id: DeviceId, addr: Addr) -> Result<(), ControllerError<T>> {
        if self.nodes.contains_key(&id) || self.dev.id() == id {
            return Err(ControllerError::DeviceIdInUse);
        }

        self.nodes.insert(id, (addr, Instant::now()));
        Ok(())
    }

    fn receive(&mut self) -> Result<(), ControllerError<T>> {
        let mut buf = [0u8; 1024];
        let (n, addr) = match self.transport.recv(&mut buf) {
            Ok((n, addr)) => (n, addr),
            Err(e) => return Err(ControllerError::TransportError(e)),
        };

        if n == 0 {
            return Ok(());
        }

        match MessageType::from_buf(&buf) {
            Some(MessageType::Hello) => {
                if let Some(DiscoveryMessage::Hello(node_id)) = DiscoveryMessage::from_bytes(&buf) {
                    self.add_node(node_id, addr)?;
                } else {
                    return Err(ControllerError::InvalidMessage);
                }
            }
            Some(MessageType::Data) => {
                let msg = DataMessage::from_bytes(&buf);
                if msg.is_none() {
                    return Err(ControllerError::InvalidMessage);
                }

                // TODO: self.handle_data(msg)
            }
            Some(MessageType::Welcome) | None => return Err(ControllerError::InvalidMessage),
            Some(MessageType::Heartbeat) => {
                let msg = match HeartbeatMessage::from_bytes(&buf) {
                    Some(msg) => msg,
                    None => return Err(ControllerError::InvalidMessage),
                };

                let node_id = msg.from();
                if let Some((_, last_seen)) = self.nodes.get_mut(&node_id) {
                    *last_seen = Instant::now();
                } else {
                    return Err(ControllerError::DeviceNotRegistered);
                }
            }
        };

        Ok(())
    }

    /// Send a [DataMessage] to a node.
    pub fn send(&mut self, msg: DataMessage, node_id: DeviceId) -> Result<(), ControllerError<T>> {
        let addr = match self.nodes.get(&node_id) {
            Some((addr, _)) => *addr,
            None => return Err(ControllerError::DeviceNotRegistered),
        };

        let raw = msg.to_bytes();

        match self.transport.send(&raw, addr) {
            Ok(()) => Ok(()),
            Err(e) => Err(ControllerError::TransportError(e)),
        }
    }
}
