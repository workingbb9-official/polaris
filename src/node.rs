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

use crate::device::Device;
use crate::protocol::{DataMessage, DiscoveryMessage, HeartbeatMessage, MessageType};
use crate::transport::{Addr, Transport};

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

/// The errors that can occur within a [Node].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeError<T: Transport> {
    /// The node is not connected to a controller, which is necessary for the operation.
    NotConnected,
    /// There was an error sending or receiving over the transport. This holds the specific error
    /// from the [Transport] trait.
    TransportError(T::Error),
    /// The message received was invalid. This could be returned if the bytes could not be parsed
    /// into a message object, or the message type was invalid for the current state.
    InvalidMessage,
    /// The message received was sent from somebody that was not the connected controller. The
    /// message is dropped for security.
    WrongController,
}

/// A device which reports to a controller.
///
/// Nodes will remain simple and able to represent any type of device. They can be anything that
/// operates independently and sends information to a controller.
pub struct Node<T: Transport, A: NodeApp> {
    dev: Device,
    controller: Option<Addr>,
    transport: T,
    app: A,
}

impl<T: Transport, A: NodeApp> Node<T, A> {
    /// Create a new node.
    pub fn new(dev: Device, transport: T, app: A) -> Self {
        Self {
            dev,
            controller: None,
            transport,
            app,
        }
    }

    /// Extract the [Device] of the node.
    #[inline]
    pub fn dev(&self) -> Device {
        self.dev
    }

    pub fn receive(&mut self) -> Result<Option<NodeEvent>, NodeError<T>> {
        let mut buf = [0u8; 260];
        let (n, addr) = match self.transport.recv(&mut buf) {
            Ok((n, addr)) => (n, addr),
            Err(e) => return Err(NodeError::TransportError(e)),
        };

        if n == 0 {
            return Ok(None);
        }

        self.handle_msg(&buf, addr)
    }

    fn handle_msg(&mut self, buf: &[u8], addr: Addr) -> Result<Option<NodeEvent>, NodeError<T>> {
        let Some(msg_type) = MessageType::from_buf(buf) else {
            return Err(NodeError::InvalidMessage);
        };

        match self.controller {
            None => {
                if msg_type != MessageType::Welcome {
                    return Err(NodeError::InvalidMessage);
                }

                if let Some(DiscoveryMessage::Welcome) = DiscoveryMessage::from_bytes(buf) {
                    self.controller = Some(addr);
                } else {
                    return Err(NodeError::InvalidMessage);
                }

                self.app.on_connection();
                Ok(Some(NodeEvent::ControllerConnected))
            }
            Some(con_addr) => {
                if addr != con_addr {
                    return Err(NodeError::WrongController);
                }

                if msg_type != MessageType::Data {
                    return Err(NodeError::InvalidMessage);
                }

                let Some(msg) = DataMessage::from_bytes(buf) else {
                    return Err(NodeError::InvalidMessage);
                };

                self.app.on_data(msg.payload());
                Ok(Some(NodeEvent::DataReceived))
            }
        }
    }

    /// Send a [DataMessage] to the controller.
    ///
    /// # Errors
    /// * `Err(NodeError::NotConnected)` - There is no controller to send data to.
    /// * `Err(NodeError::TransportError(e))` - There was an error sending the data over the wire.
    pub fn send_data(&mut self, msg: DataMessage) -> Result<(), NodeError<T>> {
        let Some(controller) = self.controller else {
            return Err(NodeError::NotConnected);
        };

        let raw = msg.to_bytes();

        match self.transport.send(&raw, controller) {
            Ok(()) => Ok(()),
            Err(e) => Err(NodeError::TransportError(e)),
        }
    }

    /// Send a heartbeat to the controller.
    ///
    /// # Errors
    /// * `Err(NodeError::NotConnected)` - There is no controller to send heartbeat to.
    /// * `Err(NodeError::TransportError(e))` - There was an error sending the heartbeat over the
    ///   wire.
    pub fn send_heartbeat(&mut self) -> Result<(), NodeError<T>> {
        let Some(controller) = self.controller else {
            return Err(NodeError::NotConnected);
        };

        let msg = HeartbeatMessage::new(self.dev.id());
        let raw = msg.to_bytes();

        match self.transport.send(&raw, controller) {
            Ok(()) => Ok(()),
            Err(e) => Err(NodeError::TransportError(e)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::device::{DeviceId, DeviceType};

    #[derive(Debug, PartialEq, Eq)]
    struct MockTransport;

    impl Transport for MockTransport {
        type Error = bool;

        fn broadcast_addr() -> Addr {
            mock_addr()
        }

        fn send(&mut self, _buf: &[u8], _addr: Addr) -> Result<(), Self::Error> {
            Ok(())
        }

        fn recv(&mut self, buf: &mut [u8]) -> Result<(usize, Addr), Self::Error> {
            let addr = mock_addr();
            Ok((buf.len(), addr))
        }
    }

    #[derive(Debug, PartialEq, Eq)]
    struct MockApp {
        data: [u8; 260],
    }

    impl NodeApp for MockApp {
        fn on_data(&mut self, data: &[u8]) {
            let len = data.len().min(260);
            self.data[..len].copy_from_slice(&data[..len]);
        }

        fn on_connection(&mut self) {}
    }

    fn mock_addr() -> Addr {
        Addr {
            octets: [127, 0, 0, 1],
            port: 8080,
        }
    }

    fn new_mock_node() -> Node<MockTransport, MockApp> {
        let app = MockApp { data: [0u8; 260] };
        let dev = Device::new(DeviceId::new(10), DeviceType::new(0));
        let mut node = Node::new(dev, MockTransport, app);

        let msg = DiscoveryMessage::new_welcome();
        let raw = msg.to_bytes();
        let addr = mock_addr();

        node.handle_msg(&raw, addr).unwrap();
        node
    }

    #[test]
    fn test_handle_data() {
        let mut node = new_mock_node();
        let msg = DataMessage::new(DeviceId::new(11), b"turn_on");
        let raw = msg.to_bytes();
        let addr = mock_addr();

        let ret = node.handle_msg(&raw, addr);
        assert_eq!(ret, Ok(Some(NodeEvent::DataReceived)));
        assert_eq!(node.app.data[..7], b"turn_on".to_vec());
    }
}
