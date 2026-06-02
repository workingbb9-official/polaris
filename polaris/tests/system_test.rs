// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 workingbb9-official

use heapless::Vec;
use zerocopy::IntoBytes;

use polaris::RegistryError;
use polaris::{Device, DeviceId, DeviceType};
use polaris::{Node, NodeAction, NodeError, NodeEvent};

fn dev(id: u16) -> Device {
    Device::new(DeviceId::new(id), DeviceType::new(20))
}

#[test]
fn peer_discovery_registers_peer_and_returns_welcome_action() {
    let local = dev(1);
    let remote = dev(2);

    let mut node: Node<&str, 8> = Node::new(local, 1000);
    let peer: Node<&str, 8> = Node::new(remote, 500);

    let hello = peer.create_hello();
    let result = node.process_msg(hello.as_bytes(), "192.168.0.22", 10);

    assert!(result.is_ok());

    let (event, action) = result.unwrap();

    match event {
        Some(NodeEvent::PeerDiscovered(found)) => {
            assert_eq!(found, remote);
        }
        _ => panic!("expected PeerDiscovered"),
    }

    match action {
        Some(NodeAction::SendWelcome { dev, msg }) => {
            assert_eq!(dev, remote);
            assert_eq!(msg[0], 0x02); // Welcome byte prefix
        }
        _ => panic!("expected SendWelcome"),
    }

    assert_eq!(node.addr(remote.id()), Some(&"192.168.0.22"));
}

#[test]
fn duplicate_peer_registration_is_rejected() {
    let local = dev(1);

    let mut node: Node<&str, 8> = Node::new(local, 1000);

    let hello = node.create_hello();
    node.process_msg(hello.as_bytes(), "peer-a", 0)
        .expect("first registration should succeed");

    let err = node
        .process_msg(hello.as_bytes(), "peer-b", 1)
        .expect_err("duplicate peer should fail");

    assert_eq!(err, NodeError::Registry(RegistryError::DeviceIdInUse));
}

#[test]
fn node_schedules_heartbeat_for_connected_peer() {
    let local = dev(1);
    let remote = dev(2);

    let mut node: Node<&str, 8> = Node::new(local, 1000);
    let peer: Node<&str, 8> = Node::new(remote, 2000);

    let hello = peer.create_hello();
    node.process_msg(hello.as_bytes(), "peer", 0)
        .expect("registration should succeed");

    let mut events: Vec<NodeEvent, 8> = Vec::new();
    let mut actions: Vec<NodeAction, 8> = Vec::new();

    node.tick(1500, &mut events, &mut actions);

    assert!(events.is_empty());
    assert_eq!(actions.len(), 1);

    match &actions[0] {
        NodeAction::SendHeartbeat { dev, msg } => {
            assert_eq!(*dev, remote);
            assert_eq!(msg[0], 0x04); // Heartbeat byte prefix
        }
        _ => panic!("expected SendHeartbeat"),
    }
}

#[test]
fn node_times_out_peer_after_missing_heartbeats() {
    let local = dev(1);
    let remote = dev(2);

    let mut node: Node<&str, 8> = Node::new(local, 1000);
    let peer: Node<&str, 8> = Node::new(remote, 500);

    let hello = peer.create_hello();
    node.process_msg(hello.as_bytes(), "peer", 0)
        .expect("registration should succeed");

    let mut events: Vec<NodeEvent, 8> = Vec::new();
    let mut actions: Vec<NodeAction, 8> = Vec::new();

    // Assumes peer timeout after ~3 missed intervals.
    node.tick(4000, &mut events, &mut actions);

    assert_eq!(events.len(), 1);

    match &events[0] {
        NodeEvent::PeerTimedOut(dev) => {
            assert_eq!(*dev, remote);
        }
        _ => panic!("expected PeerTimedOut"),
    }

    assert!(node.addr(remote.id()).is_none());
}

#[test]
fn multiple_peers_receive_heartbeat_actions() {
    let local = dev(1);

    let mut node: Node<&str, 8> = Node::new(local, 1000);

    for id in 2..=4 {
        let peer: Node<&str, 8> = Node::new(dev(id), 2000);

        let hello = peer.create_hello();
        node.process_msg(hello.as_bytes(), "peer", 0)
            .expect("registration should succeed");
    }

    let mut events: Vec<NodeEvent, 8> = Vec::new();
    let mut actions: Vec<NodeAction, 8> = Vec::new();

    node.tick(1500, &mut events, &mut actions);

    assert!(events.is_empty());
    assert_eq!(actions.len(), 3);

    for action in actions {
        match action {
            NodeAction::SendHeartbeat { msg, .. } => {
                assert_eq!(msg[0], 0x04); // Heartbeat byte prefix
            }
            _ => panic!("expected heartbeat action"),
        }
    }
}

#[test]
fn invalid_message_type_is_rejected() {
    let local = dev(1);

    let mut node: Node<&str, 8> = Node::new(local, 1000);

    let raw = [0xFF, 0xAA, 0xBB];

    let err = node
        .process_msg(&raw, "peer", 0)
        .expect_err("invalid packet should fail");

    assert_eq!(err, NodeError::InvalidMessage);
}

#[test]
fn empty_message_is_rejected() {
    let local = dev(1);

    let mut node: Node<&str, 8> = Node::new(local, 1000);

    let err = node
        .process_msg(&[], "peer", 0)
        .expect_err("empty message should fail");

    assert_eq!(err, NodeError::InvalidMessage);
}
