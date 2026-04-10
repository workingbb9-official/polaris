use crate::protocol::DeviceId;

/// Errors returned by [Controller].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControllerError {
    /// The Controller contains 'max_nodes' already. No more can be added.
    MaxNodesReached,
    /// The [DeviceId] given is already used by a node.
    DeviceIdInUse,
}

/// The main orchestrator of the system.
///
/// All devices communicate through a Controller. Main logic will be decided here, allowing the
/// nodes to stay simple and do their specific job. Because it is more complex, it is recommended
/// for Controller to be running on something that is more resource-rich in order to stay responsive
/// while maintaining the coordination of the nodes. Should be mutable to add nodes.
pub struct Controller {
    id: DeviceId,
    nodes: Vec<DeviceId>,
    max_nodes: usize,
}

impl Controller {
    /// Creates a new Controller.
    pub fn new(id: DeviceId, max_nodes: usize) -> Self {
        Self {
            id,
            nodes: Vec::with_capacity(max_nodes),
            max_nodes,
        }
    }

    /// Adds the [DeviceId] of a node to the internal Controller list.
    ///
    /// # Errors
    ///
    /// * Returns [ControllerError::MaxNodesReached] if # of stored nodes is at 'max_nodes' limit.
    /// * Returns [ControllerError::DeviceIdInUse] if a stored node or the Controller already has
    /// that DeviceId.
    pub fn add_node(&mut self, id: DeviceId) -> Result<(), ControllerError> {
        if self.nodes.len() >= self.max_nodes {
            return Err(ControllerError::MaxNodesReached);
        }

        if self.nodes.contains(&id) || self.id == id {
            return Err(ControllerError::DeviceIdInUse);
        }

        self.nodes.push(id);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_max_nodes() {
        let mut con = Controller::new(DeviceId::new(10), 1);
        con.add_node(DeviceId::new(7)).unwrap();

        let err = con.add_node(DeviceId::new(11));
        assert_eq!(err, Err(ControllerError::MaxNodesReached));
    }

    #[test]
    fn test_device_id_already_used() {
        let mut con = Controller::new(DeviceId::new(10), 5);
        con.add_node(DeviceId::new(7)).unwrap();

        let err = con.add_node(DeviceId::new(7));
        assert_eq!(err, Err(ControllerError::DeviceIdInUse));

        let err = con.add_node(DeviceId::new(10));
        assert_eq!(err, Err(ControllerError::DeviceIdInUse));
    }
}
