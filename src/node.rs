use crate::protocol::DeviceId;

/// Errors returned by [Node].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeError {
    /// The [Controller] connecting is not the right one.
    WrongController,
}

/// A device which reports to a [Controller].
///
/// Nodes will remain simple and able to describe any type of device. They can be anything that
/// operates independently, and sends information to a Controller. A node can only connect to one
/// Controller, and only one with the [DeviceId] chosen on initialization.
pub struct Node {
    id: DeviceId,
    controller: DeviceId,
    connected: bool,
}

impl Node {
    /// Creates a new Node.
    ///
    /// The 'controller' parameter determines what the node can connect to. Its [DeviceId] must be
    /// compatible with a [Controller] to connect to it.
    pub fn new(id: DeviceId, controller: DeviceId) -> Self {
        Self {
            id,
            controller,
            connected: false,
        }
    }

    /// Changes node state to connected.
    ///
    /// # Errors
    ///
    /// * Returns [NodeError::WrongController] if the [DeviceId] is not the expected one.
    pub fn connect(&mut self, id: DeviceId) -> Result<(), NodeError> {
        if id != self.controller {
            return Err(NodeError::WrongController);
        }

        self.connected = true;
        Ok(())
    }

    /// Extract the [DeviceId] of the node.
    pub fn id(&self) -> DeviceId {
        self.id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connect_to_wrong_controller() {
        let mut node = Node::new(DeviceId::new(10), DeviceId::new(11));
        let err = node.connect(DeviceId::new(15));

        assert_eq!(err, Err(NodeError::WrongController));
    }
}
