// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 workingbb9-official

#![allow(dead_code)]

use polaris::{Device, DeviceId, DeviceType};
use polaris::{Node, NodeAction, NodeEvent};
use wasm_bindgen::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct NodeId(usize);

struct SimNode {
    inner: polaris::Node<NodeId, 20>,
    id: NodeId,
    uptime: u32,
    events: heapless::Vec<NodeEvent, 8>,
    actions: heapless::Vec<NodeAction, 8>,
    connections: Vec<NodeId>,
}

impl serde::Serialize for SimNode {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        let mut state = s.serialize_struct("SimNode", 2)?;
        state.serialize_field("id", &self.id.0)?;
        state.serialize_field(
            "connections",
            &self.connections.iter().map(|c| c.0).collect::<Vec<_>>(),
        )?;
        state.end()
    }
}

impl SimNode {
    fn new(id: u16, interval: u32) -> Self {
        let node = Node::new(dev(id), interval);
        Self {
            inner: node,
            id: NodeId(id as usize),
            uptime: 0,
            events: heapless::Vec::new(),
            actions: heapless::Vec::new(),
            connections: Vec::new(),
        }
    }

    fn frame(&self) -> NodeFrame {
        NodeFrame {
            id: self.id.0,
            connections: self.connections.iter().map(|id| id.0).collect(),
        }
    }

    fn receive(&mut self, buf: &[u8], id: NodeId) {
        let (event, action) = self.inner.process_msg(buf, id, self.uptime).unwrap();

        if let Some(e) = event {
            if let NodeEvent::PeerDiscovered { .. } = e {
                self.connections.push(id);
            }

            println!("Event on {}: {:?}", self.id.0, e);
        }

        if let Some(a) = action {
            self.actions.push(a).expect("Buffer should be flushed");
        }
    }

    fn update(&mut self, elapsed: u32) {
        self.uptime += elapsed;
        self.inner
            .tick(self.uptime, &mut self.events, &mut self.actions);

        if let Some(e) = self.events.pop() {
            println!("Event on {}: {:?}", self.id.0, e);
        }
    }

    #[inline]
    fn pop_action(&mut self) -> Option<NodeAction> {
        self.actions.pop()
    }
}

#[derive(serde::Serialize)]
struct NodeFrame {
    id: usize,
    connections: Vec<usize>,
}

#[wasm_bindgen]
pub(crate) struct Simulation {
    nodes: Vec<SimNode>,
}

#[wasm_bindgen]
impl Simulation {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        let nodes: Vec<SimNode> = Vec::new();
        let mut sim = Self { nodes };

        sim.spawn_node();
        sim.spawn_node();
        sim
    }

    pub fn frame(&self) -> String {
        serde_json::to_string(&self.nodes).unwrap()
    }

    pub fn spawn_node(&mut self) {
        let node = SimNode::new(self.nodes.len() as u16, 1000);
        self.nodes.push(node);
    }

    pub fn tick(&mut self, time: u32) {
        let mut actions = Vec::new();

        for node in &mut self.nodes {
            node.update(time);

            if let Some(a) = node.pop_action() {
                actions.push((node.id, a));
            }
        }

        for (from, action) in actions {
            self.handle_action(from, action);
        }
    }

    fn handle_action(&mut self, from: NodeId, action: NodeAction) {
        match action {
            NodeAction::SendWelcome { dev, msg } => {
                let to = dev.id().value();
                self.nodes[to as usize].receive(&msg, from);
            }
            NodeAction::SendHeartbeat { dev, msg } => {
                let to = dev.id().value();
                self.nodes[to as usize].receive(&msg, from);
            }
        }
    }

    pub fn send_hello(&mut self, from: u32, to: u32) {
        let hello = self.nodes[from as usize].inner.create_hello();
        self.nodes[to as usize].receive(&hello, NodeId(from as usize));
    }

    pub fn send_data(&mut self, buf: &[u8], from: u32, to: u32) {
        let data = self.nodes[from as usize].inner.create_data(buf);
        self.nodes[to as usize].receive(&data, NodeId(from as usize));
    }
}

fn dev(id: u16) -> Device {
    Device::new(DeviceId::new(id), DeviceType::new(20))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_hello() {
        let mut sim = Simulation::new();
        sim.send_hello(sim.nodes[0].id.0 as u32, sim.nodes[1].id.0 as u32);

        assert!(matches!(
            sim.nodes[1].pop_action(),
            Some(NodeAction::SendWelcome { .. })
        ));
    }

    #[test]
    fn test_node_tick() {
        let mut sim = Simulation::new();
        sim.send_hello(sim.nodes[0].id.0 as u32, sim.nodes[1].id.0 as u32);

        sim.tick(10);

        // Ensure welcome packet was sent back to node 0
        assert_eq!(sim.nodes[0].connections[0], sim.nodes[1].id);
    }

    #[test]
    fn test_node_uptime() {
        let mut sim = Simulation::new();

        for node in &sim.nodes {
            assert_eq!(node.uptime, 0);
        }

        sim.tick(10);

        for node in &sim.nodes {
            assert_eq!(node.uptime, 10);
        }

        sim.spawn_node();
        assert_eq!(sim.nodes[2].uptime, 0);

        sim.tick(10);
        assert_eq!(sim.nodes[2].uptime, 10);
    }
}
