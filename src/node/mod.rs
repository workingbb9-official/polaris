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
use crate::transport::Transport;

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
pub enum NodeError<TE> {
    /// The node is not connected to a controller, which is necessary for the operation.
    NotConnected,
    /// There was an error sending or receiving over the transport. This holds the specific error
    /// from the [Transport] trait.
    Transport(TE),
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
#[derive(Debug)]
pub struct Node<T: Transport, A: NodeApp> {
    dev: Device,
    controller: Option<T::Addr>,
    discovery: DiscoveryManager,
    heartbeat: HeartbeatManager,
    transport: T,
    app: A,
}

impl<T: Transport, A: NodeApp> Node<T, A> {
    /// Create a new node.
    pub fn new(
        dev: Device,
        discovery_interval: u32,
        heartbeat_interval: u32,
        transport: T,
        app: A,
    ) -> Self {
        Self {
            dev,
            controller: None,
            discovery: DiscoveryManager::new(discovery_interval),
            heartbeat: HeartbeatManager::new(heartbeat_interval),
            transport,
            app,
        }
    }

    /// Extract the [Device] of the node.
    #[inline]
    pub fn dev(&self) -> Device {
        self.dev
    }

    /// Receive and process incoming packets, then send heartbeat or discovery as needed.
    ///
    /// # Errors
    /// * `Err(NodeError::WrongController)` - Received a packet from a different address.
    /// * `Err(NodeError::InvalidMessage)` - The message received was invalid. This could be due to
    ///   an incorrect format, or a wrong packet type for the current Node state.
    /// * `Err(NodeError::Transport(e))` - There was an error sending or receiving a packet.
    pub fn process(&mut self, now: u32) -> Result<Option<NodeEvent>, NodeError<T::Error>> {
        let event = self.receive()?;

        if let Some(addr) = self.controller {
            if self.heartbeat.action(now) == HeartbeatAction::Send {
                self.send_heartbeat(&addr)?;
            }
        } else if self.discovery.action(now) == DiscoveryAction::Broadcast {
            self.broadcast()?;
        }

        Ok(event)
    }

    /// Send a [DataMessage] to the controller.
    ///
    /// # Errors
    /// * `Err(NodeError::NotConnected)` - There is no controller to send data to.
    /// * `Err(NodeError::Transport(e))` - There was an error sending the data over the wire.
    pub fn send_data(&mut self, msg: DataMessage) -> Result<(), NodeError<T::Error>> {
        let Some(controller) = self.controller.as_ref() else {
            return Err(NodeError::NotConnected);
        };

        let raw = msg.to_bytes();

        self.transport
            .send(&raw, controller)
            .map_err(NodeError::Transport)
    }

    fn receive(&mut self) -> Result<Option<NodeEvent>, NodeError<T::Error>> {
        let mut buf = [0u8; 260];
        let (n, addr) = match self.transport.recv(&mut buf) {
            Ok((n, addr)) => (n, addr),
            Err(e) => return Err(NodeError::Transport(e)),
        };

        if n == 0 {
            return Ok(None);
        }

        self.handle_msg(&buf[..n], addr)
    }

    fn handle_msg(
        &mut self,
        buf: &[u8],
        addr: T::Addr,
    ) -> Result<Option<NodeEvent>, NodeError<T::Error>> {
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

    fn broadcast(&mut self) -> Result<(), NodeError<T::Error>> {
        let msg = DiscoveryMessage::new_hello(self.dev.id());
        let raw = msg.to_bytes();
        let addr = self.transport.broadcast_addr();

        self.transport
            .send(&raw, &addr)
            .map_err(NodeError::Transport)
    }

    fn send_heartbeat(&mut self, addr: &T::Addr) -> Result<(), NodeError<T::Error>> {
        let msg = HeartbeatMessage::new(self.dev.id());
        let raw = msg.to_bytes();

        self.transport
            .send(&raw, addr)
            .map_err(NodeError::Transport)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::device::{DeviceId, DeviceType};

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    struct MockAddr {
        pub octets: [u8; 4],
        pub port: u16,
    }

    #[derive(Debug, PartialEq, Eq)]
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

    fn new_mock_addr() -> MockAddr {
        MockAddr {
            octets: [127, 0, 0, 1],
            port: 8080,
        }
    }

    fn new_mock_node() -> Node<MockTransport, MockApp> {
        let app = MockApp { data: [0u8; 260] };
        let dev = Device::new(DeviceId::new(10), DeviceType::new(0));
        let mut node = Node::new(dev, 100, 50, MockTransport, app);

        let msg = DiscoveryMessage::new_welcome();
        let raw = msg.to_bytes();
        let addr = new_mock_addr();

        node.handle_msg(&raw, addr).unwrap();
        node
    }

    #[test]
    fn test_handle_data() {
        let mut node = new_mock_node();
        let msg = DataMessage::new(DeviceId::new(11), b"turn_on");
        let raw = msg.to_bytes();
        let addr = new_mock_addr();

        let ret = node.handle_msg(&raw, addr);
        assert_eq!(ret, Ok(Some(NodeEvent::DataReceived)));
        assert_eq!(node.app.data[..7], b"turn_on".to_vec());
    }
}
