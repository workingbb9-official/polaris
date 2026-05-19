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

type ControllerResult = Result<Option<ControllerEvent>, ControllerError>;

/// The significant events that can occur within a [Controller].
#[derive(Debug, PartialEq, Eq)]
pub enum ControllerEvent {
    /// The controller has received data from a node. 'from' is used to identify the [Device] that
    /// sent the message. 'range' contains the starting and ending index of the raw data, without
    /// the headers and up to the length specified by the packet.
    DataReceived {
        from: DeviceId,
        range: core::ops::Range<usize>,
    },
    /// A node has been discovered and added to the registry. Store this [DeviceId], because the
    /// controller will return this for context on which device has sent data.
    NodeDiscovered(DeviceId),
    /// A node has not sent a heartbeat message within the pre-determined time. It will be removed
    /// from the internal registry.
    NodeTimedOut(DeviceId),
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

    pub fn process_msg(
        &mut self,
        raw: &mut [u8],
        addr: Addr,
    ) -> Result<Option<ControllerEvent>, ControllerError> {
        match MessageType::from_buf(raw) {
            Some(MessageType::Hello) => self.process_hello(raw, addr),
            Some(MessageType::Data) => self.process_data(raw),
            Some(MessageType::Heartbeat) => self.process_heartbeat(raw),
            Some(MessageType::Welcome) | None => Err(ControllerError::InvalidMessage),
        }
    }

    /// Check node timeouts and remove if necessary.
    pub fn prune(&mut self) -> Vec<ControllerEvent> {
        let dead_nodes = self.registry.prune_nodes();
        dead_nodes
            .into_iter()
            .map(ControllerEvent::NodeTimedOut)
            .collect()
    }

    /// Get the network address of a node.
    #[inline]
    pub fn addr(&self, id: DeviceId) -> Result<&Addr, ControllerError> {
        self.registry.addr(id).map_err(ControllerError::Registry)
    }

    /// Authorize a pending node.
    ///
    /// The array returned is a welcome message that notifies the node it is now connected to the
    /// network. Once it is sent, the node will understand that it can start sending valid data
    /// messages to the controller.
    #[inline]
    pub fn authorize(
        &mut self,
        id: DeviceId,
        heartbeat_interval: std::time::Duration,
    ) -> Result<[u8; 5], ControllerError> {
        self.registry
            .add_node(id, heartbeat_interval)
            .map_err(ControllerError::Registry)?;

        let msg = DiscoveryMessage::new_welcome(
            heartbeat_interval
                .as_millis()
                .try_into()
                .unwrap_or(u32::MAX),
        );

        let mut raw = [0u8; 5];
        msg.to_bytes(&mut raw[..]);

        Ok(raw)
    }

    fn process_hello(&mut self, raw: &mut [u8], addr: Addr) -> ControllerResult {
        if let Some(DiscoveryMessage::Hello(dev)) = DiscoveryMessage::from_bytes(raw) {
            self.registry
                .add_pending(dev.id(), addr)
                .map_err(ControllerError::Registry)?;
            Ok(Some(ControllerEvent::NodeDiscovered(dev.id())))
        } else {
            Err(ControllerError::InvalidMessage)
        }
    }

    fn process_data(&mut self, raw: &mut [u8]) -> ControllerResult {
        DataMessage::from_bytes(raw).ok_or(ControllerError::InvalidMessage)?;
        let len = raw[3];
        let from = DeviceId::new(u16::from_be_bytes([raw[1], raw[2]]));

        self.registry
            .update_node(from)
            .map_err(ControllerError::Registry)?;

        let end = len + 4;
        Ok(Some(ControllerEvent::DataReceived {
            from,
            range: 4..end as usize,
        }))
    }

    fn process_heartbeat(&mut self, raw: &mut [u8]) -> ControllerResult {
        let msg = HeartbeatMessage::from_bytes(raw).ok_or(ControllerError::InvalidMessage)?;

        let node_id = msg.from();
        self.registry
            .update_node(node_id)
            .map_err(ControllerError::Registry)?;
        Ok(None)
    }
}
