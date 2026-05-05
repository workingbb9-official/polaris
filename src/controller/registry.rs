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

use std::collections::HashMap;
use std::time::Instant;

use crate::device::DeviceId;

#[derive(Debug, PartialEq)]
pub enum RegistryError {
    /// Used on discovery to represent that the [DeviceId] of the node being connected to is
    /// already in use by a stored node.
    DeviceIdInUse,
    /// The number of nodes in the registry has reached 'max_nodes', no more can be added.
    MaxNodesReached,
    /// The DeviceId of a node was not found within the registry.
    NodeNotRegistered,
}

#[derive(Debug, PartialEq)]
struct NodeEntry<A> {
    addr: A,
    seen: Instant,
}

#[derive(Debug)]
pub(crate) struct NodeRegistry<A> {
    nodes: HashMap<DeviceId, NodeEntry<A>>,
    max_nodes: usize,
}

impl<A> NodeRegistry<A> {
    pub(crate) fn new(max_nodes: usize) -> Self {
        Self {
            nodes: HashMap::new(),
            max_nodes,
        }
    }

    pub(crate) fn add_node(&mut self, id: DeviceId, addr: A) -> Result<(), RegistryError> {
        if self.nodes.contains_key(&id) {
            return Err(RegistryError::DeviceIdInUse);
        }

        if self.nodes.len() >= self.max_nodes {
            return Err(RegistryError::MaxNodesReached);
        }

        let node = NodeEntry {
            addr,
            seen: Instant::now(),
        };

        self.nodes.insert(id, node);
        Ok(())
    }

    pub(crate) fn update_node(&mut self, id: DeviceId) -> Result<(), RegistryError> {
        let entry = self
            .nodes
            .get_mut(&id)
            .ok_or(RegistryError::NodeNotRegistered)?;
        entry.seen = Instant::now();
        Ok(())
    }

    pub(crate) fn addr(&self, id: DeviceId) -> Result<&A, RegistryError> {
        let entry = self
            .nodes
            .get(&id)
            .ok_or(RegistryError::NodeNotRegistered)?;
        Ok(&entry.addr)
    }
}
