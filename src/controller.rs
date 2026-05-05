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

mod registry;

use registry::{NodeRegistry, RegistryError};

use crate::Transport;
use crate::device::{Device, DeviceId};
use crate::protocol::{DataMessage, DiscoveryMessage, HeartbeatMessage, MessageType};

/// The significant events that can occur within a [Controller].
#[derive(Debug, PartialEq, Eq)]
pub enum ControllerEvent {
    /// The controller has received data from a node. Use the [DataMessage] method `.from()` to
    /// determine the identity of the node.
    DataReceived(Box<DataMessage>),
    /// A node has been discovered and added to the registry. Store this [DeviceId], because the
    /// controller will return this for context on which device has sent data.
    NodeDiscovered(DeviceId),
}

/// The errors that can occur within a [Controller].
#[derive(Debug, PartialEq)]
pub enum ControllerError<T: Transport> {
    /// Error sending or receiving, holds the specific error from the Transport trait.
    Transport(T::Error),
    /// The message received was invalid. This could be returned if the bytes could not be parsed
    /// into a 'Message' object, or the message type was invalid for a controller to receive.
    InvalidMessage,
    /// There was an error within the registry. Error is held and propagated.
    Registry(RegistryError),
}

/// The main orchestrator of the system.
///
/// All devices communicate through a Controller. Main logic will be decided here, allowing the
/// nodes to stay simple and do their specific job. Because it is more complex, it is recommended
/// for a Controller to be a device that has more resources in order to stay responsive while
/// maintaining the coordination of the nodes.
pub struct Controller<T: Transport> {
    dev: Device,
    registry: NodeRegistry<T::Addr>,
    transport: T,
}

impl<T: Transport> Controller<T> {
    /// Create a new controller.
    pub fn new(dev: Device, max_nodes: usize, transport: T) -> Self {
        Self {
            dev,
            registry: NodeRegistry::new(max_nodes),
            transport,
        }
    }

    /// Extract the [Device] of the controller.
    #[inline]
    pub fn dev(&self) -> Device {
        self.dev
    }

    pub fn receive(&mut self) -> Result<Option<ControllerEvent>, ControllerError<T>> {
        let mut buf = [0u8; 1024];
        let (n, addr) = match self.transport.recv(&mut buf) {
            Ok((n, addr)) => (n, addr),
            Err(e) => return Err(ControllerError::Transport(e)),
        };

        if n == 0 {
            return Ok(None);
        }

        self.handle_msg(&buf, addr)
    }

    fn handle_msg(
        &mut self,
        buf: &[u8],
        addr: T::Addr,
    ) -> Result<Option<ControllerEvent>, ControllerError<T>> {
        match MessageType::from_buf(buf) {
            Some(MessageType::Hello) => {
                if let Some(DiscoveryMessage::Hello(node_id)) = DiscoveryMessage::from_bytes(buf) {
                    self.registry
                        .add_node(node_id, addr)
                        .map_err(ControllerError::Registry)?;
                    Ok(Some(ControllerEvent::NodeDiscovered(node_id)))
                } else {
                    Err(ControllerError::InvalidMessage)
                }
            }
            Some(MessageType::Data) => {
                let msg = DataMessage::from_bytes(buf).ok_or(ControllerError::InvalidMessage)?;

                Ok(Some(ControllerEvent::DataReceived(Box::new(msg))))
            }
            Some(MessageType::Heartbeat) => {
                let msg =
                    HeartbeatMessage::from_bytes(buf).ok_or(ControllerError::InvalidMessage)?;

                let node_id = msg.from();
                self.registry
                    .update_node(node_id)
                    .map_err(ControllerError::Registry)?;
                Ok(None)
            }
            Some(MessageType::Welcome) | None => Err(ControllerError::InvalidMessage),
        }
    }

    /// Send a [DataMessage] to a node.
    ///
    /// # Errors
    /// * `Err(ControllerError::DeviceNotRegistered) - The [DeviceId] of the node is not within the
    ///   registry.
    /// * `Err(ControllerError::Transport(e)) - There was an error sending the data over the
    ///   wire.
    pub fn send(&mut self, msg: DataMessage, node_id: DeviceId) -> Result<(), ControllerError<T>> {
        let addr = self
            .registry
            .addr(node_id)
            .map_err(ControllerError::Registry)?;
        let raw = msg.to_bytes();

        self.transport
            .send(&raw, addr)
            .map_err(ControllerError::Transport)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::device::{DeviceId, DeviceType};

    #[derive(Debug, Clone, Copy, PartialEq)]
    struct MockAddr {
        pub octets: [u8; 4],
        pub port: u16,
    }

    #[derive(Debug, PartialEq)]
    struct MockTransport;
    impl Transport for MockTransport {
        type Addr = MockAddr;
        type Error = bool;

        fn broadcast_addr(&mut self) -> Self::Addr {
            new_mock_addr()
        }

        fn send(&mut self, _buf: &[u8], _addr: &Self::Addr) -> Result<(), Self::Error> {
            Ok(())
        }

        fn recv(&mut self, buf: &mut [u8]) -> Result<(usize, Self::Addr), Self::Error> {
            let addr = new_mock_addr();
            Ok((buf.len(), addr))
        }
    }

    fn new_mock_addr() -> MockAddr {
        MockAddr {
            octets: [127, 0, 0, 1],
            port: 8080,
        }
    }

    fn new_mock_controller(max_nodes: usize) -> Controller<MockTransport> {
        let dev = Device::new(DeviceId::new(10), DeviceType::new(0));
        let mut con = Controller::new(dev, max_nodes, MockTransport);

        let addr = new_mock_addr();
        con.registry.add_node(DeviceId::new(11), addr).unwrap();

        con
    }

    #[test]
    fn test_handle_discovery() {
        let mut con = new_mock_controller(7);
        let msg = DiscoveryMessage::new_hello(DeviceId::new(13));
        let raw = msg.to_bytes();
        let addr = new_mock_addr();

        let ret = con.handle_msg(&raw, addr);
        assert_eq!(
            ret,
            Ok(Some(ControllerEvent::NodeDiscovered(DeviceId::new(13))))
        );
    }

    #[test]
    fn test_handle_data() {
        let mut con = new_mock_controller(7);
        let msg = DataMessage::new(DeviceId::new(13), b"hello");
        let raw = msg.to_bytes();
        let addr = new_mock_addr();

        let ret = con.handle_msg(&raw, addr);
        assert_eq!(ret, Ok(Some(ControllerEvent::DataReceived(Box::new(msg)))));

        match ret {
            Ok(Some(ControllerEvent::DataReceived(msg))) => {
                assert_eq!(msg.from(), DeviceId::new(13));
                assert_eq!(msg.payload(), b"hello");
            }
            _ => panic!(),
        };
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
        assert_eq!(
            ret,
            Err(ControllerError::Registry(RegistryError::NodeNotRegistered))
        );
    }
}
