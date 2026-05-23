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
    /// Used on discovery to represent that the [DeviceId] of the peer being connected to is
    /// already in use by a stored peer.
    DeviceIdInUse,
    /// The number of peers in the registry has reached 'max_peers', no more can be added.
    MaxPeersReached,
    /// The DeviceId of a peer was not found within the registry.
    PeerNotRegistered,
}

#[derive(Debug, PartialEq)]
struct Peer<Addr> {
    dev: Device,
    addr: Addr,
    last_seen: Instant,
    timeout: Duration,
    last_sent: Instant,
    send_interval: Duration,
}

#[derive(Debug)]
pub(crate) struct PeerRegistry<Addr> {
    peers: HashMap<DeviceId, Peer<Addr>>,
    pending: HashMap<DeviceId, Peer<Addr>>,
    max_peers: usize,
}

impl<Addr> PeerRegistry<Addr> {
    pub(crate) fn new(max_peers: usize) -> Self {
        Self {
            peers: HashMap::new(),
            pending: HashMap::new(),
            max_peers,
        }
    }

    pub(crate) fn add_peer(
        &mut self,
        id: DeviceId,
        timeout: Duration,
        send_interval: Duration,
    ) -> Result<(), RegistryError> {
        if self.peers.contains_key(&id) {
            return Err(RegistryError::DeviceIdInUse);
        }

        if self.peers.len() >= self.max_peers {
            return Err(RegistryError::MaxPeersReached);
        }

        if let Some(mut pending) = self.pending.remove(&id) {
            pending.timeout = timeout;
            self.peers.insert(id, pending);
        } else {
            return Err(RegistryError::PeerNotRegistered);
        }

        Ok(())
    }

    pub(crate) fn update_peer(&mut self, id: DeviceId) -> Result<(), RegistryError> {
        let entry = self
            .peers
            .get_mut(&id)
            .ok_or(RegistryError::PeerNotRegistered)?;
        entry.seen = Instant::now();
        Ok(())
    }

    pub(crate) fn prune_peers(&mut self) -> Vec<DeviceId> {
        let mut dead_peers = Vec::new();

        self.peers.retain(|id, entry| {
            if Instant::now() - entry.seen >= entry.timeout {
                dead_peers.push(*id);
                false
            } else {
                true
            }
        });

        dead_peers
    }

    pub(crate) fn addr(&self, id: DeviceId) -> Result<&Addr, RegistryError> {
        let entry = self
            .peers
            .get(&id)
            .ok_or(RegistryError::PeerNotRegistered)?;
        Ok(&entry.addr)
    }

    pub(crate) fn add_pending(&mut self, dev: Device, addr: Addr) -> Result<(), RegistryError> {
        if self.peers.contains_key(&dev.id()) || self.pending.contains_key(&dev.id()) {
            return Err(RegistryError::DeviceIdInUse);
        }

        if self.peers.len() + self.pending.len() >= self.max_peers {
            return Err(RegistryError::MaxPeersReached);
        }

        let peer = Peer {
            dev,
            addr,
            last_seen: Instant::now(),
            timeout: Duration::from_secs(0),
            last_sent: Instant::now(),
            send_interval: Duration::from_secs(0),
        };

        self.pending.insert(dev.id(), peer);
        Ok(())
    }

    pub(crate) fn peers(&self) -> &HashMap<DeviceId, Peer<Addr>> {
        &self.peers
    }
}
