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
    /// into a 'Message' object, or the message type was invalid for a controller to receive.
    InvalidMessage,
    /// The [DeviceId] received was not found within the Controller registry.
    DeviceNotRegistered,
    /// The amount of nodes in the registry has reached 'max_nodes', no more can be added.
    MaxNodesReached,
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
    /// Create a new controller.
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
    /// * `Err(ControllerError::DeviceNotRegistered)` - The [DeviceId] was not found in the registry.
    pub fn last_seen(&self, id: DeviceId) -> Result<Instant, ControllerError<T>> {
        match self.nodes.get(&id) {
            Some((_, last_seen)) => Ok(*last_seen),
            None => Err(ControllerError::DeviceNotRegistered),
        }
    }

    /// Add a node to the registry.
    ///
    /// # Errors
    /// * `Err(ControllerError::DeviceIdInUse)` - The [DeviceId] being added is already in the
    ///   registry.
    /// * `Err(ControllerError::MaxNodesReached)` - The controller cannot add any more nodes to the
    ///   registry.
    pub fn add_node(&mut self, id: DeviceId, addr: Addr) -> Result<(), ControllerError<T>> {
        if self.nodes.contains_key(&id) || self.dev.id() == id {
            return Err(ControllerError::DeviceIdInUse);
        }

        if self.nodes.len() >= self.max_nodes {
            return Err(ControllerError::MaxNodesReached);
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

        self.handle_msg(&buf, addr)
    }

    fn handle_msg(&mut self, buf: &[u8], addr: Addr) -> Result<(), ControllerError<T>> {
        match MessageType::from_buf(buf) {
            Some(MessageType::Hello) => {
                if let Some(DiscoveryMessage::Hello(node_id)) = DiscoveryMessage::from_bytes(buf) {
                    self.add_node(node_id, addr)?;
                } else {
                    return Err(ControllerError::InvalidMessage);
                }
            }
            Some(MessageType::Data) => {
                let msg = DataMessage::from_bytes(buf);
                if msg.is_none() {
                    return Err(ControllerError::InvalidMessage);
                }

                // TODO: self.handle_data(msg);
            }
            Some(MessageType::Heartbeat) => {
                let msg = match HeartbeatMessage::from_bytes(buf) {
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
            Some(MessageType::Welcome) | None => return Err(ControllerError::InvalidMessage),
        };

        Ok(())
    }

    /// Send a [DataMessage] to a node.
    ///
    /// # Errors
    /// * `Err(ControllerError::DeviceNotRegistered) - The [DeviceId] of the node is not within the
    ///   registry.
    /// * `Err(ControllerError::TransportError(e)) - There was an error sending the data over the
    ///   wire.
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::device::DeviceType;

    #[derive(Debug, PartialEq, Eq)]
    struct MockTransport;
    impl Transport for MockTransport {
        type Error = bool;

        fn send(&mut self, _buf: &[u8], _addr: Addr) -> Result<(), Self::Error> {
            Ok(())
        }

        fn recv(&mut self, buf: &mut [u8]) -> Result<(usize, Addr), Self::Error> {
            let addr = mock_addr();
            Ok((buf.len(), addr))
        }
    }

    fn mock_addr() -> Addr {
        Addr {
            octets: [127, 0, 0, 1],
            port: 8080,
        }
    }

    fn new_mock_controller(max_nodes: usize) -> Controller<MockTransport> {
        let dev = Device::new(DeviceId::new(10), DeviceType::new(0));
        let mut con = Controller::new(dev, max_nodes, MockTransport);

        let addr = mock_addr();
        con.add_node(DeviceId::new(11), addr).unwrap();

        con
    }

    #[test]
    fn test_handle_discovery() {
        let mut con = new_mock_controller(7);
        let msg = DiscoveryMessage::new_hello(DeviceId::new(13));
        let raw = msg.to_bytes();
        let addr = mock_addr();

        let ret = con.handle_msg(&raw, addr);
        assert_eq!(ret, Ok(()));
        assert_eq!(con.nodes.contains_key(&DeviceId::new(13)), true);
    }

    #[test]
    fn test_handle_heartbeat() {
        let mut con = new_mock_controller(7);
        let msg = HeartbeatMessage::new(DeviceId::new(11));
        let raw = msg.to_bytes();
        let addr = mock_addr();
        let start_time = Instant::now();

        let ret = con.handle_msg(&raw, addr);
        assert_eq!(ret, Ok(()));

        let seen = con.last_seen(DeviceId::new(11)).unwrap();
        assert!(seen >= start_time);

        let now = Instant::now();
        assert!(seen <= now);
    }

    #[test]
    fn test_send_data() {
        let mut con = new_mock_controller(7);
        let msg = DataMessage::new(DeviceId::new(10), b"hello");

        let ret = con.send(msg, DeviceId::new(11));
        assert_eq!(ret, Ok(()));
    }

    #[test]
    fn test_send_to_unregistered_node() {
        let mut con = new_mock_controller(7);
        let msg = DataMessage::new(DeviceId::new(10), b"hello");

        let ret = con.send(msg, DeviceId::new(5));
        assert_eq!(ret, Err(ControllerError::DeviceNotRegistered));
    }
}
