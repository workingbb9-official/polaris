use polaris::{Device, DeviceId, DeviceType};
use polaris::{Node, NodeAction, NodeEvent};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NodeId(pub usize);

fn dev(id: u16) -> Device {
    Device::new(DeviceId::new(id), DeviceType::new(20))
}

pub struct SimNode {
    pub inner: polaris::Node<NodeId, 20>,
    pub id: NodeId,
    pub uptime: u32,
    pub peers: Vec<NodeId>,
    events: heapless::Vec<NodeEvent, 8>,
    actions: heapless::Vec<NodeAction, 8>,
}

impl serde::Serialize for SimNode {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        let mut state = s.serialize_struct("SimNode", 3)?;
        state.serialize_field("id", &self.id.0)?;
        state.serialize_field("uptime", &self.uptime)?;
        state.serialize_field("peers", &self.peers.iter().map(|c| c.0).collect::<Vec<_>>())?;
        state.end()
    }
}

impl SimNode {
    pub fn new(id: u16, interval: u32) -> Self {
        let node = Node::new(dev(id), interval);
        Self {
            inner: node,
            id: NodeId(id as usize),
            uptime: 0,
            events: heapless::Vec::new(),
            actions: heapless::Vec::new(),
            peers: Vec::new(),
        }
    }

    pub fn receive(&mut self, buf: &[u8], id: NodeId) {
        let (event, action) = match self.inner.process_msg(buf, id, self.uptime) {
            Ok((e, a)) => (e, a),
            Err(_) => return,
        };

        if let Some(e) = event {
            if let NodeEvent::PeerDiscovered { .. } = e {
                self.peers.push(id);
            }

            println!("Event on {}: {:?}", self.id.0, e);
        }

        if let Some(a) = action {
            self.actions.push(a).expect("Buffer should be flushed");
        }
    }

    pub fn update(&mut self, elapsed: u32) {
        self.uptime += elapsed;
        self.inner
            .tick(self.uptime, &mut self.events, &mut self.actions);

        while let Some(e) = self.events.pop() {
            if let NodeEvent::PeerTimedOut(dev) = e {
                self.peers.retain(|id| id.0 as u16 != dev.id().value());
            }

            println!("Event on {}: {:?}", self.id.0, e);
        }
    }

    #[inline]
    pub fn pop_action(&mut self) -> Option<NodeAction> {
        self.actions.pop()
    }
}
