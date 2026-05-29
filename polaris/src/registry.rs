// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 workingbb9-official

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

    pub fn update_peer_seen(&mut self, id: DeviceId, now: u32) -> Result<(), RegistryError> {
        let Some(peer) = self.peers.iter_mut().find(|peer| peer.dev.id() == id) else {
            return Err(RegistryError::PeerNotRegistered);
        };

        peer.update_last_seen(now);
        Ok(())
    }

    pub fn update_peer_sent(&mut self, id: DeviceId, now: u32) -> Result<(), RegistryError> {
        let Some(peer) = self.peers.iter_mut().find(|peer| peer.dev.id() == id) else {
            return Err(RegistryError::PeerNotRegistered);
        };

        peer.update_last_sent(now);
        Ok(())
    }

    pub fn dead_peers(&mut self, now: u32) -> Vec<Device, MAX_PEERS> {
        self.peers
            .iter()
            .filter(|peer| peer.is_timed_out(now))
            .map(|peer| peer.dev)
            .collect()
    }

    pub fn pending_heartbeats(&self, now: u32, heartbeat_interval: u32) -> Vec<Device, MAX_PEERS> {
        self.peers
            .iter()
            .filter(|peer: &&Peer<Addr>| peer.needs_heartbeat(now, heartbeat_interval))
            .map(|peer| peer.dev)
            .collect()
    }

    pub fn addr(&self, id: DeviceId) -> Option<&Addr> {
        self.peers
            .iter()
            .find(|peer| peer.dev.id() == id)
            .map(|peer| &peer.addr)
    }

    pub fn addr_mut(&mut self, id: DeviceId) -> Option<&mut Addr> {
        self.peers
            .iter_mut()
            .find(|peer| peer.dev.id() == id)
            .map(|peer| &mut peer.addr)
    }

    pub fn remove(&mut self, id: DeviceId) {
        self.peers.retain(|peer| peer.dev.id() != id);
    }
}
