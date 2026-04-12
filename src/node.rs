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

use crate::device::DeviceId;
use crate::protocol::RawMessage;

/// Errors returned by [Node].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeError {
    /// The `Controller` connecting is not the right one.
    WrongController,
}

/// A device which reports to a `Controller`.
///
/// Nodes will remain simple and able to describe any type of device. They can be anything that
/// operates independently, and sends information to a Controller. A node can only connect to one
/// Controller, and only one with the [DeviceId] chosen on initialization.
pub struct Node {
    id: DeviceId,
    controller_id: DeviceId,
    connected: bool,
}

impl Node {
    /// Creates a new Node.
    ///
    /// The `controller_id` parameter determines what the node can connect to. Its [DeviceId] must
    /// be compatible with a `Controller` to connect to it.
    pub fn new(id: DeviceId, controller_id: DeviceId) -> Self {
        Self {
            id,
            controller_id,
            connected: false,
        }
    }

    /// Changes node state to connected.
    ///
    /// # Errors
    ///
    /// * Returns [NodeError::WrongController] if the [DeviceId] is not the expected one.
    pub fn connect(&mut self, id: DeviceId) -> Result<(), NodeError> {
        if id != self.controller_id {
            return Err(NodeError::WrongController);
        }

        self.connected = true;
        Ok(())
    }

    /// Extract the [DeviceId] of the node.
    #[inline]
    pub fn id(&self) -> DeviceId {
        self.id
    }

    /// Construct and send a [RawMessage] packet.
    pub fn send(&self, payload: [u8; 256]) -> Result<(), NodeError> {
        let _msg = RawMessage::new(self.id, self.controller_id, &payload);
        todo!("Implement UDP and send 'msg'");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connect_to_wrong_controller() {
        let mut node = Node::new(DeviceId::new(10), DeviceId::new(11));
        let err = node.connect(DeviceId::new(15));

        assert_eq!(err, Err(NodeError::WrongController));
    }
}
