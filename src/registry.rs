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

use crate::device::{Device, DeviceId};
use crate::peer::Peer;
use heapless::Vec;

#[derive(Debug, PartialEq)]
pub enum RegistryError {
    /// The [DeviceId] of a peer is already within the registry.
    DeviceIdInUse,
    /// The peer could not be found within the registry.
    PeerNotRegistered,
    /// Cannot add any more peers because the registry is full.
    MaxPeersReached,
}

#[derive(Debug)]
pub(crate) struct PeerRegistry<Addr, const MAX_PEERS: usize> {
    peers: Vec<Peer<Addr>, MAX_PEERS>,
}

impl<Addr, const MAX_PEERS: usize> PeerRegistry<Addr, MAX_PEERS> {
    pub fn new() -> Self {
        Self { peers: Vec::new() }
    }

    pub fn add_peer(&mut self, peer: Peer<Addr>) -> Result<(), RegistryError> {
        let id = peer.dev.id();

        if self.peers.iter().any(|peer| peer.dev.id() == id) {
            return Err(RegistryError::DeviceIdInUse);
        }

        self.peers
            .push(peer)
            .map_err(|_| RegistryError::MaxPeersReached)
    }

    pub fn update_peer(&mut self, id: DeviceId, now: u32) -> Result<(), RegistryError> {
        let Some(peer) = self.peers.iter_mut().find(|peer| peer.dev.id() == id) else {
            return Err(RegistryError::PeerNotRegistered);
        };

        peer.update_last_seen(now);
        Ok(())
    }

    pub fn dead_peers(&mut self, now: u32) -> Vec<Device, MAX_PEERS> {
        self.peers
            .iter()
            .filter(|peer| peer.is_timed_out(now))
            .map(|peer| peer.dev)
            .collect()
    }

    pub fn pending_heartbeats(&self, now: u32) -> Vec<Device, MAX_PEERS> {
        self.peers
            .iter()
            .filter(|peer: &&Peer<Addr>| peer.needs_heartbeat(now))
            .map(|peer| peer.dev)
            .collect()
    }

    pub fn addr(&self, id: DeviceId) -> Option<&Addr> {
        self.peers
            .iter()
            .find(|peer| peer.dev.id() == id)
            .map(|peer| &peer.addr)
    }

    pub fn remove(&mut self, id: DeviceId) {
        self.peers.retain(|peer| peer.dev.id() != id);
    }
}
