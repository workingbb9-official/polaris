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
    dev: Device,
    nodes: Vec<DeviceId>,
    max_nodes: usize,
}

impl Controller {
    /// Creates a new Controller.
    pub fn new(dev: Device, max_nodes: usize) -> Self {
        Self {
            dev,
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

        if self.nodes.contains(&dev.id()) || self.dev.id() == dev.id() {
            return Err(ControllerError::DeviceIdInUse);
        }

        self.nodes.push(dev.id());
        Ok(())
    }

    pub fn dev(&self) -> Device {
        self.dev
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::device::DeviceType;

    #[test]
    fn test_max_nodes() {
        let con_dev = Device::new(DeviceId::new(10), DeviceType::new(0));
        let mut con = Controller::new(con_dev, 1);

        let dev = Device::new(DeviceId::new(7), DeviceType::new(13));
        con.add_node(dev).unwrap();

        let dev = Device::new(DeviceId::new(11), DeviceType::new(15));
        let err = con.add_node(dev);

        assert_eq!(err, Err(ControllerError::MaxNodesReached));
    }

    #[test]
    fn test_device_id_already_used() {
        let con_dev = Device::new(DeviceId::new(10), DeviceType::new(0));
        let mut con = Controller::new(con_dev, 5);

        let dev = Device::new(DeviceId::new(11), DeviceType::new(13));
        con.add_node(dev).unwrap();

        let dev = Device::new(DeviceId::new(10), DeviceType::new(7));
        let err = con.add_node(dev);

        assert_eq!(err, Err(ControllerError::DeviceIdInUse));

        let dev = Device::new(DeviceId::new(11), DeviceType::new(11));
        let err = con.add_node(dev);

        assert_eq!(err, Err(ControllerError::DeviceIdInUse));
    }
}
