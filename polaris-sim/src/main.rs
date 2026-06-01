use polaris::{Device, DeviceId, DeviceType};
use polaris::{Node, NodeAction, NodeEvent};

fn main() {
    let mut sim = Simulation::new();
    sim.connect(sim.nodes[0].id, sim.nodes[1].id);
    sim.tick(10);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct NodeId(usize);

#[derive(Debug)]
struct SimNode {
    inner: polaris::Node<NodeId, 20>,
    id: NodeId,
    connections: Vec<NodeId>,
    uptime: u32,
}

impl SimNode {
    fn new(id: u16, interval: u32) -> Self {
        let node = Node::new(dev(id), interval);
        Self {
            inner: node,
            id: NodeId(id as usize),
            connections: Vec::new(),
            uptime: 0,
        }
    }

    fn receive(&mut self, buf: &[u8], id: NodeId) -> Option<NodeAction> {
        match self.inner.process_msg(buf, id, self.uptime).unwrap() {
            (Some(NodeEvent::PeerDiscovered(_)), Some(action)) => {
                self.connections.push(id);
                Some(action)
            }
            (Some(NodeEvent::PeerDiscovered(_)), None) => {
                self.connections.push(id);
                None
            }
            (_, Some(action)) => Some(action),
            (_, None) => None,
        }
    }
}

#[derive(Debug)]
struct Simulation {
    nodes: Vec<SimNode>,
}

impl Simulation {
    fn new() -> Self {
        let mut nodes: Vec<SimNode> = Vec::new();

        for i in 0..5 {
            let sim_node = SimNode::new(i as u16, 1000);
            nodes.push(sim_node);
        }

        Self { nodes }
    }

    fn connect(&mut self, node1: NodeId, node2: NodeId) {
        let hello = self.nodes[node1.0].inner.create_hello();

        if let Some(NodeAction::SendWelcome { msg, .. }) =
            self.nodes[node2.0].receive(&hello, node1)
        {
            self.nodes[node1.0].receive(&msg, node2);
        }
    }

    fn tick(&mut self, time: u32) {
        for node in &mut self.nodes {
            node.uptime += time;
        }
    }
}

fn dev(id: u16) -> Device {
    Device::new(DeviceId::new(id), DeviceType::new(20))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connect_nodes() {
        let mut sim = Simulation::new();
        sim.connect(sim.nodes[0].id, sim.nodes[1].id);

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

        sim.nodes.push(SimNode::new(6, 1000));
        assert_eq!(sim.nodes[5].uptime, 0);

        sim.tick(10);
        assert_eq!(sim.nodes[5].uptime, 10);
    }
}
