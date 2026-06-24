use crate::sim_node::{NodeId, SimNode};

use polaris::DATA_HEADER_LEN;
use polaris::NodeAction;

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

        for node in &mut self.nodes {
            node.update(time);

            while let Some(a) = node.pop_action() {
                actions.push((node.id, a));
            }
        }

        let mut sim_events = Vec::new();

        for (from, action) in actions {
            sim_events.push(self.handle_action(from, action));
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

                // TODO: make heartbeat packet enum variant
                SimEvent::WelcomePacketSent {
                    from_x,
                    from_y,
                    to_x,
                    to_y,
                }
            }
        }
    }

    pub fn send_hello(&mut self, from: u32, to: u32) {
        let hello = self.nodes[from as usize].inner.create_hello();
        self.nodes[to as usize].receive(&hello, NodeId(from as usize));
    }

    pub fn send_data(&mut self, buf: &[u8], from: u32, to: u32) {
        let mut packet = Vec::new();
        let len = DATA_HEADER_LEN + buf.len();

        self.nodes[from as usize]
            .inner
            .create_data(buf, &mut packet[..len])
            .unwrap();

        self.nodes[to as usize].receive(&packet, NodeId(from as usize));
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
        sim.tick(10);

        // Ensure welcome packet was sent back to node 0
        assert_eq!(sim.nodes[0].peers[0], sim.nodes[1].id);
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

        assert_eq!(sim.nodes[0].peers, Vec::new());

        sim.spawn_node(0, 0, 100);
        sim.send_hello(sim.nodes[1].id.0 as u32, sim.nodes[0].id.0 as u32);
        assert_eq!(sim.nodes[0].peers[0], sim.nodes[1].id);

        // Timeout occurs before heartbeat can be sent
        sim.tick(400);
        assert_eq!(sim.nodes[0].peers, Vec::new());
    }
}
