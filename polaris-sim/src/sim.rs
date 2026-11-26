use crate::sim_node::{NodeId, SimNode};

use polaris::DATA_HEADER_LEN;
use polaris::{NodeAction, NodeEvent};

use serde::Serialize;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct Simulation {
    nodes: Vec<SimNode>,
}

#[derive(Serialize)]
#[serde(tag = "type")]
pub enum SimEvent {
    WelcomePacketSent {
        from_x: u32,
        from_y: u32,
        to_x: u32,
        to_y: u32,
    },
    HeartbeatPacketSent {
        from_x: u32,
        from_y: u32,
        to_x: u32,
        to_y: u32,
    },
    PeerDiscovered {
        node: u32,
        peer: u32,
    },
    PeerTimedOut {
        node: u32,
        peer: u32,
    },
    DataReceived {
        node: u32,
        peer: u32,
        data: Vec<u8>,
    },
}

#[wasm_bindgen]
impl Simulation {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self { nodes: Vec::new() }
    }

    pub fn frame(&self) -> String {
        serde_json::to_string(&self.nodes).unwrap()
    }

    pub fn node_info(&self, id: u32) -> String {
        serde_json::to_string(&self.nodes[id as usize]).unwrap()
    }

    pub fn node_position(&self, id: u32) -> Result<Vec<u32>, JsValue> {
        if self.nodes.len() as u32 <= id {
            return Err(JsValue::from_str("Index out of bounds"));
        }

        let (x, y) = self.nodes[id as usize].position();
        Ok(vec![x, y])
    }

    pub fn total_nodes(&self) -> u32 {
        self.nodes.len() as u32
    }

    // Node IDs are auto incremented starting from 0
    pub fn spawn_node(&mut self, x: u32, y: u32, heartbeat: u32) {
        let node = SimNode::new(self.nodes.len() as u16, x, y, heartbeat);
        self.nodes.push(node);
    }

    pub fn tick(&mut self, time: u32) -> String {
        let mut actions = Vec::new();
        let mut events = Vec::new();

        for node in &mut self.nodes {
            node.update(time);

            while let Some(action) = node.pop_action() {
                actions.push((node.id, action));
            }

            while let Some(event) = node.pop_event() {
                events.push((node.id, event));
            }
        }

        let mut sim_events = Vec::new();

        for (from, action) in actions {
            sim_events.push(self.handle_action(from, action));
        }

        for (from, event) in events {
            sim_events.push(self.handle_event(from, event));
        }

        serde_json::to_string(&sim_events).unwrap()
    }

    fn handle_action(&mut self, from: NodeId, action: NodeAction) -> SimEvent {
        match action {
            NodeAction::SendWelcome { dev, msg } => {
                let to = dev.id().value();
                self.nodes[to as usize].receive(&msg, from);

                let (from_x, from_y) = self.nodes[from.0].position();
                let (to_x, to_y) = self.nodes[to as usize].position();

                SimEvent::WelcomePacketSent {
                    from_x,
                    from_y,
                    to_x,
                    to_y,
                }
            }
            NodeAction::SendHeartbeat { dev, msg } => {
                let to = dev.id().value();
                self.nodes[to as usize].receive(&msg, from);

                let (from_x, from_y) = self.nodes[from.0].position();
                let (to_x, to_y) = self.nodes[to as usize].position();

                let uptime = self.nodes[from.0].uptime;
                self.nodes[from.0].inner.msg_sent(dev.id(), uptime).unwrap();

                SimEvent::HeartbeatPacketSent {
                    from_x,
                    from_y,
                    to_x,
                    to_y,
                }
            }
        }
    }

    fn handle_event(&mut self, from: NodeId, event: NodeEvent) -> SimEvent {
        match event {
            NodeEvent::PeerDiscovered(dev) => {
                let peer = dev.id().value() as u32;

                SimEvent::PeerDiscovered {
                    node: from.0 as u32,
                    peer,
                }
            }
            NodeEvent::PeerTimedOut(dev) => {
                let peer = dev.id().value() as u32;

                SimEvent::PeerTimedOut {
                    node: from.0 as u32,
                    peer,
                }
            }
            NodeEvent::DataReceived { from: peer_id, .. } => {
                let peer = peer_id.value() as u32;
                let data: Vec<u8> = self.nodes[from.0].pop_data().expect("No data in queue");

                SimEvent::DataReceived {
                    node: from.0 as u32,
                    peer,
                    data,
                }
            }
        }
    }

    /// Sends a hello packet to initiate a connection
    pub fn send_hello(&mut self, from: u32, to: u32) {
        let hello = self.nodes[from as usize].inner.create_hello();
        self.nodes[to as usize].receive(&hello, NodeId(from as usize));
    }

    /// Sends a data packet containing `buf`
    pub fn send_data(&mut self, buf: &[u8], from: u32, to: u32) {
        let len = DATA_HEADER_LEN + buf.len();
        let mut packet = vec![0u8; len];

        self.nodes[from as usize]
            .inner
            .create_data(buf, &mut packet)
            .unwrap();

        self.nodes[to as usize].receive(&packet, NodeId(from as usize));

        let id = self.nodes[to as usize].inner.dev().id();
        let uptime = self.nodes[from as usize].uptime;
        self.nodes[from as usize]
            .inner
            .msg_sent(id, uptime)
            .unwrap();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_hello() {
        let mut sim = Simulation::new();
        sim.spawn_node(0, 0, 100);
        sim.spawn_node(0, 0, 100);

        sim.send_hello(sim.nodes[0].id.0 as u32, sim.nodes[1].id.0 as u32);

        assert!(matches!(
            sim.nodes[1].pop_action(),
            Some(NodeAction::SendWelcome { .. })
        ));
    }

    #[test]
    fn test_node_tick() {
        let mut sim = Simulation::new();
        sim.spawn_node(0, 0, 100);
        sim.spawn_node(0, 0, 100);

        sim.send_hello(sim.nodes[0].id.0 as u32, sim.nodes[1].id.0 as u32);
        let events = sim.tick(10);

        assert_eq!(sim.nodes[0].peers()[0], sim.nodes[1].id);

        let json: Vec<serde_json::Value> = serde_json::from_str(&events).unwrap();

        assert!(json.contains(&serde_json::json!({
            "type": "WelcomePacketSent",
            "from_x": 0,
            "from_y": 0,
            "to_x": 0,
            "to_y": 0,
        })));

        assert!(json.contains(&serde_json::json!({
            "type": "PeerDiscovered",
            "node": 1,
            "peer": 0,
        })));
    }

    #[test]
    fn test_node_uptime() {
        let mut sim = Simulation::new();
        sim.spawn_node(0, 0, 100);

        assert_eq!(sim.nodes[0].uptime, 0);

        sim.tick(10);
        assert_eq!(sim.nodes[0].uptime, 10);

        sim.spawn_node(0, 0, 100);
        assert_eq!(sim.nodes[1].uptime, 0);

        sim.tick(10);
        assert_eq!(sim.nodes[1].uptime, 10);
    }

    #[test]
    fn test_peer_list() {
        let mut sim = Simulation::new();
        sim.spawn_node(0, 0, 100);

        assert_eq!(sim.nodes[0].peers().len(), 0);

        sim.spawn_node(0, 0, 100);
        sim.send_hello(sim.nodes[1].id.0 as u32, sim.nodes[0].id.0 as u32);
        assert_eq!(sim.nodes[0].peers()[0], sim.nodes[1].id);

        // Timeout occurs before heartbeat can be sent
        sim.tick(400);
        assert_eq!(sim.nodes[0].peers().len(), 0);
    }

    #[test]
    fn test_data_transfer() {
        let mut sim = Simulation::new();
        sim.spawn_node(0, 0, 100);
        sim.spawn_node(0, 0, 100);

        // Connect nodes
        sim.send_hello(sim.nodes[0].id.0 as u32, sim.nodes[1].id.0 as u32);
        sim.tick(10);
        sim.tick(10);

        sim.send_data(
            b"Roblox",
            sim.nodes[0].id.0 as u32,
            sim.nodes[1].id.0 as u32,
        );

        let events = sim.tick(10);
        let json: Vec<serde_json::Value> = serde_json::from_str(&events).unwrap();

        assert!(json.contains(&serde_json::json!({
            "type": "DataReceived",
            "node": 1,
            "peer": 0,
            "data": b"Roblox",
        })));
    }
}
