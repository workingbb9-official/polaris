use polaris::{Device, DeviceId, DeviceType};
use polaris::{Node, NodeAction, NodeEvent};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NodeId(pub usize);

fn dev(id: u16) -> Device {
    Device::new(DeviceId::new(id), DeviceType::new(20))
}

pub struct SimNode {
    pub inner: Node<NodeId, 20>,
    pub id: NodeId,
    pub uptime: u32,

    events: heapless::Vec<NodeEvent, 8>,
    actions: heapless::Vec<NodeAction, 8>,

    position: (u32, u32),
}

impl serde::Serialize for SimNode {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;

        let mut state = serializer.serialize_struct("SimNode", 3)?;

        state.serialize_field("id", &self.id.0)?;
        state.serialize_field("uptime", &self.uptime)?;

        let peers: Vec<u32> = self.peers().into_iter().map(|id| id.0 as u32).collect();
        state.serialize_field("peers", &peers)?;
        state.end()
    }
}

impl SimNode {
    pub fn new(id: u16, x: u32, y: u32, interval: u32) -> Self {
        Self {
            inner: Node::new(dev(id), interval),
            id: NodeId(id as usize),
            uptime: 0,
            events: heapless::Vec::new(),
            actions: heapless::Vec::new(),
            position: (x, y),
        }
    }

    /// Returns X and Y position of node
    pub fn position(&self) -> (u32, u32) {
        self.position
    }

    /// Returns peers as [`NodeId`]
    pub fn peers(&self) -> Vec<NodeId> {
        let peers: Vec<NodeId> = self
            .inner
            .peers()
            .map(|peer| NodeId(peer.dev.id().value() as usize))
            .collect();
        peers
    }

    /// Processes message and stores outputted events and actions to take.
    pub fn receive(&mut self, buf: &[u8], from: NodeId) {
        let Ok((event, action)) = self.inner.process_msg(buf, from, self.uptime) else {
            return;
        };

        if let Some(event) = event {
            self.events.push(event).expect("Event buffer full");
        }

        if let Some(action) = action {
            self.actions.push(action).expect("Action buffer full");
        }
    }

    /// Updates node uptime and ticks time forward, collecting events and actions to take.
    pub fn update(&mut self, elapsed: u32) {
        self.uptime += elapsed;

        self.inner
            .tick(self.uptime, &mut self.events, &mut self.actions);
    }

    #[allow(dead_code)]
    #[inline]
    pub fn pop_event(&mut self) -> Option<NodeEvent> {
        self.events.pop()
    }

    #[inline]
    pub fn pop_action(&mut self) -> Option<NodeAction> {
        self.actions.pop()
    }
}
