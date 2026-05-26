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
use crate::protocol::{HeartbeatMessage, HelloMessage, MessageType, Packet, WelcomeMessage};
use crate::registry::{PeerRegistry, RegistryError};

use heapless::Vec;
use zerocopy::{FromBytes, IntoBytes};

/// The significant events that can occur within a [Node].
#[derive(Debug, PartialEq, Eq)]
pub enum NodeEvent {
    /// The node has received data from a peer. 'from' is used to identify the [Device] that
    /// sent the message. 'range' contains the starting and ending index of the raw data, without
    /// the headers and up to the length specified by the packet.
    DataReceived {
        from: DeviceId,
        range: core::ops::Range<usize>,
    },
    /// A peer has been discovered and added to the pending registry. Store this [Device] and use
    /// its [DeviceId] to authorize the peer so that it can begin sending data.  The DeviceId will
    /// also be returned by the library to identify where the data is coming from.
    PeerDiscovered(Device),
    /// A peer has not sent a heartbeat message within the pre-determined time. It will be removed
    /// from the internal registry.
    PeerTimedOut(Device),
}

#[derive(Debug)]
pub enum NodeAction {
    /// Send out a heartbeat message to a peer. This ensures that the peer does not disconnect
    /// from the node. The [DeviceId] within 'dev' can be used to find the address of the peer.
    SendHeartbeat { dev: Device, msg: [u8; 3] },
    /// Send a welcome message to a peer. This allows them to collect the information of this node
    /// to their registry, establishing the connection. Use the [DeviceId] within 'dev' to find the
    /// address of the peer.
    SendWelcome { dev: Device, msg: [u8; 9] },
}

/// The errors that can occur within a [Node].
#[derive(Debug, PartialEq)]
pub enum NodeError {
    /// The message received was invalid. This could be returned if the bytes could not be parsed
    /// into a 'Message' object, or the message type was invalid for a node to receive.
    InvalidMessage,
    /// Error propagated from the registry.
    Registry(RegistryError),
}

#[derive(Debug)]
pub struct Node<Addr, const MAX_PEERS: usize> {
    dev: Device,
    registry: PeerRegistry<Addr, MAX_PEERS>,
    heartbeat_interval: u32,
}

impl<Addr: core::fmt::Debug, const MAX_PEERS: usize> Node<Addr, MAX_PEERS> {
    /// Create a new node.
    ///
    /// The heartbeat interval determines how often this node will send heartbeats to its peers. If
    /// the peers miss 3 heartbeats in a row, they will drop this node.
    pub fn new(dev: Device, heartbeat_interval: u32) -> Self {
        Self {
            dev,
            registry: PeerRegistry::new(),
            heartbeat_interval,
        }
    }

    /// Extract the [Device] of the node.
    #[inline]
    pub fn dev(&self) -> Device {
        self.dev
    }

    /// Get the addr of a peer by its [DeviceId].
    #[inline]
    pub fn addr(&self, id: DeviceId) -> Option<&Addr> {
        self.registry.addr(id)
    }

    /// Update the time last sent to a peer.
    ///
    /// Call this when either a data message or a heartbeat message is sent. This keeps both nodes
    /// synchronized, so there are no redundant heartbeats sent.
    #[inline]
    pub fn msg_sent(&mut self, id: DeviceId, now: u32) -> Result<(), RegistryError> {
        self.registry.update_peer_sent(id, now)
    }

    /// Collect passive events and actions to take.
    ///
    /// This will push pending actions and events into provided buffers. The size of these buffers
    /// should be determined based on system memory and level of importance.
    pub fn tick<const E: usize, const A: usize>(
        &mut self,
        now: u32,
        out_events: &mut Vec<NodeEvent, E>,
        out_actions: &mut Vec<NodeAction, A>,
    ) {
        for dev in self.registry.dead_peers(now) {
            if out_events.is_full() {
                break;
            } else {
                out_events
                    .push(NodeEvent::PeerTimedOut(dev))
                    .expect("Already checked if vector was full");
                self.registry.remove(dev.id());
            }
        }

        for dev in self
            .registry
            .pending_heartbeats(now, self.heartbeat_interval)
        {
            if out_actions.is_full() {
                break;
            } else {
                let msg = HeartbeatMessage {
                    from: self.dev.id(),
                };
                let packet = Packet::new(MessageType::Heartbeat, msg);

                out_actions
                    .push(NodeAction::SendHeartbeat {
                        dev,
                        msg: packet
                            .as_bytes()
                            .try_into()
                            .expect("Heartbeat message is 2 bytes (DeviceId)"),
                    })
                    .expect("Already checked if vector was full");
            }
        }
    }

    pub fn process_msg(
        &mut self,
        raw: &[u8],
        addr: Addr,
        now: u32,
    ) -> Result<(Option<NodeEvent>, Option<NodeAction>), NodeError> {
        if raw.is_empty() {
            return Err(NodeError::InvalidMessage);
        }

        let msg_type = match MessageType::try_from(raw[0]) {
            Ok(t) => t,
            Err(_) => return Err(NodeError::InvalidMessage),
        };

        match msg_type {
            MessageType::Hello => {
                if let Ok(packet) = Packet::<HelloMessage>::ref_from_bytes(raw) {
                    let hello = &packet.payload;
                    self.process_hello(hello, addr, now)
                        .map(|(a, b)| (Some(a), Some(b)))
                } else {
                    Err(NodeError::InvalidMessage)
                }
            }
            MessageType::Heartbeat => {
                if let Ok(packet) = Packet::<HeartbeatMessage>::ref_from_bytes(raw) {
                    let heartbeat = &packet.payload;
                    self.process_heartbeat(heartbeat, now).map(|_| (None, None))
                } else {
                    Err(NodeError::InvalidMessage)
                }
            }
            _ => todo!(),
        }
    }

    fn process_hello(
        &mut self,
        msg: &HelloMessage,
        addr: Addr,
        now: u32,
    ) -> Result<(NodeEvent, NodeAction), NodeError> {
        let peer = Peer::new(msg.dev, addr, now, msg.heartbeat_interval);
        self.registry.add_peer(peer).map_err(NodeError::Registry)?;

        let welcome = WelcomeMessage {
            dev: self.dev,
            heartbeat_interval: self.heartbeat_interval,
        };
        let packet = Packet::new(MessageType::Welcome, welcome);

        let event = NodeEvent::PeerDiscovered(msg.dev);

        let action = NodeAction::SendWelcome {
            dev: msg.dev,
            msg: packet
                .as_bytes()
                .try_into()
                .expect("Welcome message is 8 bytes (4 for the interval, 4 for Device"),
        };

        Ok((event, action))
    }

    fn process_heartbeat(&mut self, msg: &HeartbeatMessage, now: u32) -> Result<(), NodeError> {
        self.registry
            .update_peer_seen(msg.from, now)
            .map_err(NodeError::Registry)?;
        Ok(())
    }

    /*    /// Return an optional action to take based on current state.
    pub fn action(&mut self, now: u32) -> Option<NodeAction<Addr>> {
        if let Some(addr) = self.controller {
            if self.heartbeat.action(now) == HeartbeatAction::Send {
                let msg = HeartbeatMessage::new(self.dev.id());
                let raw = msg.to_bytes();
                Some(NodeAction::SendHeartbeat { addr, msg: raw })
            } else {
                None
            }
        } else if self.discovery.action(now) == DiscoveryAction::Broadcast {
            let msg = DiscoveryMessage::new_hello(self.dev);
            let mut raw = [0u8; 3];
            msg.to_bytes(&mut raw[..]);
            Some(NodeAction::SendDiscovery { msg: raw })
        } else {
            None
        }
    }

    pub fn process_msg(
        &mut self,
        raw: &mut [u8],
        addr: Addr,
    ) -> Result<Option<NodeEvent>, NodeError> {
        match MessageType::from_buf(raw) {
            Some(MessageType::Hello) => self.process_hello(raw, addr),
            Some(MessageType::Data) => self.process_data(raw),
            Some(MessageType::Heartbeat) => self.process_heartbeat(raw),
            Some(MessageType::Welcome) | None => Err(NodeError::InvalidMessage),
        }
    }

    /// Check peer timeouts and remove if necessary.
    pub fn prune(&mut self) -> Vec<NodeEvent> {
        let dead_peers = self.registry.prune_peers();
        dead_peers
            .into_iter()
            .map(NodeEvent::PeerTimedOut)
            .collect()
    }

    /// Get the network address of a peer.
    #[inline]
    pub fn addr(&self, id: DeviceId) -> Result<&Addr, NodeError> {
        self.registry.addr(id).map_err(NodeError::Registry)
    }

    /// Authorize a pending peer.
    ///
    /// The array returned is a welcome message that notifies the peer it is now connected to the
    /// network. Once it is sent, the peer will understand that it can start sending valid data
    /// messages to this node.
    #[inline]
    pub fn authorize(
        &mut self,
        id: DeviceId,
        heartbeat_interval: std::time::Duration,
    ) -> Result<[u8; 5], NodeError> {
        self.registry
            .add_peer(id, heartbeat_interval)
            .map_err(NodeError::Registry)?;

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

    fn process_hello(&mut self, raw: &mut [u8], addr: Addr) -> NodeResult {
        if let Some(DiscoveryMessage::Hello(dev)) = DiscoveryMessage::from_bytes(raw) {
            self.registry
                .add_pending(dev, addr)
                .map_err(NodeError::Registry)?;
            Ok(Some(NodeEvent::PeerDiscovered(dev)))
        } else {
            Err(NodeError::InvalidMessage)
        }
    }

    fn process_data(&mut self, raw: &mut [u8]) -> NodeResult {
        DataMessage::from_bytes(raw).ok_or(NodeError::InvalidMessage)?;
        let len = raw[3];
        let from = DeviceId::new(u16::from_be_bytes([raw[1], raw[2]]));

        self.registry
            .update_peer(from)
            .map_err(NodeError::Registry)?;

        let end = len + 4;
        Ok(Some(NodeEvent::DataReceived {
            from,
            range: 4..end as usize,
        }))
    }

    fn process_heartbeat(&mut self, raw: &mut [u8]) -> NodeResult {
        let msg = HeartbeatMessage::from_bytes(raw).ok_or(NodeError::InvalidMessage)?;

        let peer_id = msg.from();
        self.registry
            .update_peer(peer_id)
            .map_err(NodeError::Registry)?;
        Ok(None)
    } */
}
