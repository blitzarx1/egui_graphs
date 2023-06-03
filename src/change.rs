use egui::Vec2;
use petgraph::stable_graph::{EdgeIndex, NodeIndex};

/// `ChangeNode` is a enum that stores the changes to `Node` properties.
#[derive(Debug, Clone)]
pub enum ChangeNode {
    /// Node has been clicked
    Clicked { id: NodeIndex },

    /// Node has been clicked
    DoubleClicked { id: NodeIndex },

    /// Node has changed its location
    Location { id: NodeIndex, old: Vec2, new: Vec2 },

    /// Node has been selected or deselected
    Selected { id: NodeIndex, old: bool, new: bool },

    /// Node has been folded or unfolded
    Folded { id: NodeIndex, old: bool, new: bool },

    /// Node is dragged or ceased to be dragged
    Dragged { id: NodeIndex, old: bool, new: bool },
}

impl ChangeNode {
    pub(crate) fn clicked(id: NodeIndex) -> Self {
        Self::Clicked { id }
    }

    pub(crate) fn double_clicked(id: NodeIndex) -> Self {
        Self::DoubleClicked { id }
    }

    pub(crate) fn change_location(id: NodeIndex, old: Vec2, new: Vec2) -> Self {
        Self::Location { id, old, new }
    }

    pub(crate) fn change_selected(id: NodeIndex, old: bool, new: bool) -> Self {
        Self::Selected { id, old, new }
    }

    pub(crate) fn change_folded(id: NodeIndex, old: bool, new: bool) -> Self {
        Self::Folded { id, old, new }
    }

    pub(crate) fn change_dragged(id: NodeIndex, old: bool, new: bool) -> Self {
        Self::Dragged { id, old, new }
    }
}

/// `ChangeEdge` is a enum that stores the changes to `Edge` properties.
#[derive(Debug, Clone)]
pub enum ChangeEdge {
    Selected { id: EdgeIndex, old: bool, new: bool },
}

impl ChangeEdge {
    pub(crate) fn change_selected(id: EdgeIndex, old: bool, new: bool) -> Self {
        Self::Selected { id, old, new }
    }
}

/// Change is a enum that stores the changes to `Node` or `Edge` properties.
#[derive(Debug, Clone)]
pub enum Change {
    Node(ChangeNode),
    Edge(ChangeEdge),
}

impl Change {
    pub(crate) fn node(change: ChangeNode) -> Self {
        Self::Node(change)
    }

    pub(crate) fn edge(change: ChangeEdge) -> Self {
        Self::Edge(change)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::vec2;

    #[test]
    fn test_change_enum() {
        let node_id = NodeIndex::new(0);
        let edge_id = EdgeIndex::new(0);

        let old_node_location = vec2(0.0, 0.0);
        let new_node_location = vec2(10.0, 10.0);

        let node_change = Change::node(ChangeNode::change_location(
            node_id,
            old_node_location,
            new_node_location,
        ));

        let node_selected_old = false;
        let node_selected_new = true;

        let edge_change = Change::edge(ChangeEdge::change_selected(
            edge_id,
            node_selected_old,
            node_selected_new,
        ));

        match node_change {
            Change::Node(ChangeNode::Location { id, old, new }) => {
                assert_eq!(id, node_id);
                assert_eq!(old, old_node_location);
                assert_eq!(new, new_node_location);
            }
            _ => panic!("Unexpected node change type"),
        }

        match edge_change {
            Change::Edge(ChangeEdge::Selected { id, old, new }) => {
                assert_eq!(id, edge_id);
                assert_eq!(old, node_selected_old);
                assert_eq!(new, node_selected_new);
            }
            _ => panic!("Unexpected edge change type"),
        }
    }
}
