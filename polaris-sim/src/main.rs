use polaris::{Device, DeviceId, DeviceType};
use polaris::{Node, NodeAction, NodeEvent};

fn main() {
    let mut sim = Simulation::new();
    sim.tick(10);

    let node1_id = sim.nodes[0].id;

    let hello = sim.nodes[0].inner.create_hello();
    sim.nodes[1].receive(&hello, node1_id);

    if let Some(NodeAction::SendWelcome { msg, .. }) = sim.nodes[1].pop_action() {
        let node2_id = sim.nodes[1].id;

        sim.nodes[0].receive(&msg, node2_id);
        assert!(matches!(
            sim.nodes[0].pop_event(),
            Some(NodeEvent::PeerDiscovered { .. })
        ));
        assert_eq!(sim.nodes[0].connections[0], node2_id);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct NodeId(usize);

#[derive(Debug)]
struct SimNode {
    inner: polaris::Node<NodeId, 20>,
    id: NodeId,
    uptime: u32,
    events: heapless::Vec<NodeEvent, 8>,
    actions: heapless::Vec<NodeAction, 8>,
    connections: Vec<NodeId>,
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

    // Return value represents if event or action was pushed
    fn receive(&mut self, buf: &[u8], id: NodeId) -> (bool, bool) {
        let mut ev = false;
        let mut act = false;

        let (event, action) = self.inner.process_msg(buf, id, self.uptime).unwrap();

        if let Some(e) = event {
            if let NodeEvent::PeerDiscovered(_) = e {
                self.connections.push(id);
            }

            self.events.push(e).expect("Buffer should be flushed");
            ev = true;
        }

        if let Some(a) = action {
            self.actions.push(a).expect("Buffer should be flushed");
            act = true;
        }

        (ev, act)
    }

    #[inline]
    fn pop_event(&mut self) -> Option<NodeEvent> {
        self.events.pop()
    }

    #[inline]
    fn pop_action(&mut self) -> Option<NodeAction> {
        self.actions.pop()
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
    fn test_node_hello() {
        let mut sim = Simulation::new();
        let node1_id = sim.nodes[0].id;

        let hello = sim.nodes[0].inner.create_hello();
        sim.nodes[1].receive(&hello, node1_id);

        assert!(matches!(
            sim.nodes[1].pop_action(),
            Some(NodeAction::SendWelcome { .. })
        ));

        assert!(matches!(
            sim.nodes[1].pop_event(),
            Some(NodeEvent::PeerDiscovered { .. })
        ));
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
