// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 workingbb9-official

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
}
