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

use crate::device::{Device, DeviceId};

/// Errors returned by [Controller].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControllerError {
    /// The Controller contains 'max_nodes' already. No more can be added.
    MaxNodesReached,
    /// The [DeviceId] given is already used by a node.
    DeviceIdInUse,
}

/// The main orchestrator of the system.
///
/// All devices communicate through a Controller. Main logic will be decided here, allowing the
/// nodes to stay simple and do their specific job. Because it is more complex, it is recommended
/// for a Controller to be a device that has more resources in order to stay responsive while
/// maintaining the coordination of the nodes.
pub struct Controller {
    id: DeviceId,
    nodes: Vec<DeviceId>,
    max_nodes: usize,
}

impl Controller {
    /// Creates a new Controller.
    pub fn new(id: DeviceId, max_nodes: usize) -> Self {
        Self {
            id,
            nodes: Vec::with_capacity(max_nodes),
            max_nodes,
        }
    }

    /// Adds the [DeviceId] of a node to the internal Controller list.
    ///
    /// # Errors
    ///
    /// * Returns [ControllerError::MaxNodesReached] if # of stored nodes is at 'max_nodes' limit.
    /// * Returns [ControllerError::DeviceIdInUse] if a stored node or the Controller itself
    ///   already has that DeviceId.
    pub fn add_node(&mut self, dev: Device) -> Result<(), ControllerError> {
        if self.nodes.len() >= self.max_nodes {
            return Err(ControllerError::MaxNodesReached);
        }

        if self.nodes.contains(&dev.id()) || self.id == dev.id() {
            return Err(ControllerError::DeviceIdInUse);
        }

        self.nodes.push(dev.id());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_max_nodes() {
        let mut con = Controller::new(DeviceId::new(10), 1);
        con.add_node(DeviceId::new(7)).unwrap();

        let err = con.add_node(DeviceId::new(11));
        assert_eq!(err, Err(ControllerError::MaxNodesReached));
    }

    #[test]
    fn test_device_id_already_used() {
        let mut con = Controller::new(DeviceId::new(10), 5);
        con.add_node(DeviceId::new(7)).unwrap();

        let err = con.add_node(DeviceId::new(7));
        assert_eq!(err, Err(ControllerError::DeviceIdInUse));

        let err = con.add_node(DeviceId::new(10));
        assert_eq!(err, Err(ControllerError::DeviceIdInUse));
    }
}
