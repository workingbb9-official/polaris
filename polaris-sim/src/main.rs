use polaris::{Device, DeviceId, DeviceType};
use polaris::{Node, NodeAction};

struct SimNode {
    inner: polaris::Node<usize, 20>,
    addr: usize,
}

struct Simulation {
    nodes: Vec<SimNode>,
}

fn main() {
    let mut sim = make_sim(5);

    if let [node1, node2, ..] = &mut sim.nodes[..] {
        connect(node1, node2);
    }
}

fn make_sim(total_nodes: usize) -> Simulation {
    let mut nodes: Vec<SimNode> = Vec::new();
    for i in 0..total_nodes {
        let node = Node::new(dev(i as u16), 1000);
        let sim_node = SimNode {
            inner: node,
            addr: i,
        };
        nodes.push(sim_node);
    }

    Simulation { nodes }
}

fn dev(id: u16) -> Device {
    Device::new(DeviceId::new(id), DeviceType::new(20))
}

fn connect(node1: &mut SimNode, node2: &mut SimNode) {
    let addr = node1.addr;

    let hello = node1.inner.create_hello();
    let (_, action) = node2.inner.process_msg(&hello, addr, 0).unwrap();

    let Some(NodeAction::SendWelcome { .. }) = action else {
        panic!(
            "Expected Some(NodeAction::SendWelcome), but got {:?}",
            action
        );
    };
}
