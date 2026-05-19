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
use std::time::{Duration, Instant};

use crate::device::{Device, DeviceId};

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
struct NodeEntry<Addr> {
    dev: Device,
    addr: Addr,
    seen: Instant,
    timeout: Duration,
}

#[derive(Debug)]
pub(crate) struct NodeRegistry<Addr> {
    nodes: HashMap<DeviceId, NodeEntry<Addr>>,
    pending: HashMap<DeviceId, NodeEntry<Addr>>,
    max_nodes: usize,
}

impl<Addr> NodeRegistry<Addr> {
    pub(crate) fn new(max_nodes: usize) -> Self {
        Self {
            nodes: HashMap::new(),
            pending: HashMap::new(),
            max_nodes,
        }
    }

    pub(crate) fn add_node(
        &mut self,
        id: DeviceId,
        timeout: Duration,
    ) -> Result<(), RegistryError> {
        if self.nodes.contains_key(&id) {
            return Err(RegistryError::DeviceIdInUse);
        }

        if self.nodes.len() >= self.max_nodes {
            return Err(RegistryError::MaxNodesReached);
        }

        if let Some(mut pending) = self.pending.remove(&id) {
            pending.timeout = timeout;
            self.nodes.insert(id, pending);
        } else {
            return Err(RegistryError::NodeNotRegistered);
        }

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

    pub(crate) fn prune_nodes(&mut self) -> Vec<DeviceId> {
        let mut dead_nodes = Vec::new();

        self.nodes.retain(|id, entry| {
            if Instant::now() - entry.seen >= entry.timeout {
                dead_nodes.push(*id);
                false
            } else {
                true
            }
        });

        dead_nodes
    }

    pub(crate) fn addr(&self, id: DeviceId) -> Result<&Addr, RegistryError> {
        let entry = self
            .nodes
            .get(&id)
            .ok_or(RegistryError::NodeNotRegistered)?;
        Ok(&entry.addr)
    }

    pub(crate) fn add_pending(&mut self, dev: Device, addr: Addr) -> Result<(), RegistryError> {
        if self.nodes.contains_key(&dev.id()) || self.pending.contains_key(&dev.id()) {
            return Err(RegistryError::DeviceIdInUse);
        }

        if self.nodes.len() + self.pending.len() >= self.max_nodes {
            return Err(RegistryError::MaxNodesReached);
        }

        let node = NodeEntry {
            dev,
            addr,
            seen: Instant::now(),
            timeout: Duration::from_secs(0),
        };

        self.pending.insert(dev.id(), node);
        Ok(())
    }
}
