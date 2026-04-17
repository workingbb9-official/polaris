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
use crate::{Addr, Transport};

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
pub struct Controller<T: Transport> {
    dev: Device,
    nodes: HashMap<DeviceId, (Addr, Instant)>,
    max_nodes: usize,
    transport: T,
}

impl<T: Transport> Controller<T> {
    /// Creates a new Controller.
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
}
