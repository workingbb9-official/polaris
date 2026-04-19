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
use crate::protocol::{DataMessage, DiscoveryMessage, MessageType};
use crate::transport::{Addr, Transport};

/// The errors that can occur within a [Node].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeError<T: Transport> {
    /// The node is not connected to a controller, which is necessary for the operation.
    NotConnected,
    /// There was an error sending or receiving over the transport. This holds the specific error
    /// from the Transport trait.
    TransportError(T::Error),
    /// The message received was invalid. This could be returned if the bytes could not be parsed
    /// into a 'Message' object, or the message type was invalid for a node to receive.
    InvalidMessage,
}

/// A device which reports to a controller.
///
/// Nodes will remain simple and able to represent any type of device. They can be anything that
/// operates independently and sends information to a controller.
pub struct Node<T: Transport> {
    dev: Device,
    controller: Option<Addr>,
    transport: T,
}

impl<T: Transport> Node<T> {
    /// Create a new Node.
    pub fn new(dev: Device, transport: T) -> Self {
        Self {
            dev,
            controller: None,
            transport,
        }
    }

    /// Extract the [Device] of the node.
    #[inline]
    pub fn dev(&self) -> Device {
        self.dev
    }

    fn receive(&mut self) -> Result<(), NodeError<T>> {
        let mut buf = [0u8; 260];
        let (n, addr) = match self.transport.recv(&mut buf) {
            Ok((n, addr)) => (n, addr),
            Err(e) => return Err(NodeError::TransportError(e)),
        };

        if n == 0 {
            return Ok(());
        }

        self.handle_msg(&buf, addr)
    }

    fn handle_msg(&mut self, buf: &[u8], addr: Addr) -> Result<(), NodeError<T>> {
        match MessageType::from_buf(buf) {
            Some(MessageType::Welcome) => {
                if self.controller.is_some() {
                    return Ok(());
                }

                if let Some(DiscoveryMessage::Welcome) = DiscoveryMessage::from_bytes(buf) {
                    self.controller = Some(addr);
                } else {
                    return Err(NodeError::InvalidMessage);
                }
            }
            Some(MessageType::Data) => {
                let msg = DataMessage::from_bytes(buf);
                if msg.is_none() {
                    return Err(NodeError::InvalidMessage);
                }

                // TODO: self.handle_data(msg);
            }
            Some(MessageType::Heartbeat) | Some(MessageType::Hello) | None => {
                return Err(NodeError::InvalidMessage);
            }
        };

        Ok(())
    }
    /// Send a [DataMessage] to the controller.
    ///
    /// # Errors
    /// * 'Err(NodeError::NotConnected)' - There is no controller to send data to.
    /// * 'Err(NodeError::TransportError(e))' - There was an error sending the data over the wire.
    pub fn send_data(&mut self, msg: DataMessage) -> Result<(), NodeError<T>> {
        if self.controller.is_none() {
            return Err(NodeError::NotConnected);
        }

        let raw = msg.to_bytes();

        match self.transport.send(&raw, self.controller.unwrap()) {
            Ok(()) => Ok(()),
            Err(e) => Err(NodeError::TransportError(e)),
        }
    }
}
