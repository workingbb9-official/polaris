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

use crate::device::{Device, DeviceId};
use crate::protocol::{DataMessage, DiscoveryMessage, HeartbeatMessage, MessageType};

/// The significant events that can occur within a [Controller].
#[derive(Debug, PartialEq, Eq)]
pub enum ControllerEvent<'a> {
    /// The controller has received data from a node. From is used to identify the [Device] that
    /// sent the message. The buffer returned is the exact data slice, with no headers and the exact
    /// length specified by the packet.
    DataReceived { from: DeviceId, data: &'a [u8] },
    /// A node has been discovered and added to the registry. Store this [DeviceId], because the
    /// controller will return this for context on which device has sent data.
    NodeDiscovered(DeviceId),
}

/// The errors that can occur within a [Controller].
#[derive(Debug, PartialEq)]
pub enum ControllerError {
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
pub struct Controller<Addr> {
    dev: Device,
    registry: NodeRegistry<Addr>,
}

impl<Addr> Controller<Addr> {
    /// Create a new controller.
    pub fn new(dev: Device, max_nodes: usize) -> Self {
        Self {
            dev,
            registry: NodeRegistry::new(max_nodes),
        }
    }

    /// Extract the [Device] of the controller.
    #[inline]
    pub fn dev(&self) -> Device {
        self.dev
    }

    pub fn process_msg<'a>(
        &mut self,
        raw: &'a mut [u8],
        addr: Addr,
    ) -> Result<Option<ControllerEvent<'a>>, ControllerError> {
        match MessageType::from_buf(raw) {
            Some(MessageType::Hello) => {
                if let Some(DiscoveryMessage::Hello(node_id)) = DiscoveryMessage::from_bytes(raw) {
                    self.registry
                        .add_node(node_id, addr)
                        .map_err(ControllerError::Registry)?;
                    Ok(Some(ControllerEvent::NodeDiscovered(node_id)))
                } else {
                    Err(ControllerError::InvalidMessage)
                }
            }
            Some(MessageType::Data) => {
                DataMessage::from_bytes(raw).ok_or(ControllerError::InvalidMessage)?;
                let len = raw[3];
                let from = DeviceId::new(u16::from_be_bytes([raw[1], raw[2]]));

                Ok(Some(ControllerEvent::DataReceived {
                    from,
                    data: &raw[3..len as usize],
                }))
            }
            Some(MessageType::Heartbeat) => {
                let msg =
                    HeartbeatMessage::from_bytes(raw).ok_or(ControllerError::InvalidMessage)?;

                let node_id = msg.from();
                self.registry
                    .update_node(node_id)
                    .map_err(ControllerError::Registry)?;
                Ok(None)
            }
            Some(MessageType::Welcome) | None => Err(ControllerError::InvalidMessage),
        }
    }

    #[inline]
    pub fn addr(&self, id: DeviceId) -> Result<&Addr, ControllerError> {
        self.registry.addr(id).map_err(ControllerError::Registry)
    }
}
