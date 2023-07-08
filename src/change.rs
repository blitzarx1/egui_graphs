use egui::Vec2;
use petgraph::stable_graph::{EdgeIndex, NodeIndex};

use crate::SubGraph;

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

/// `ChangeSubgraph` is a enum that stores the changes which affect a subgraph.
#[derive(Debug, Clone)]
pub enum ChangeSubgraph {
    /// Indicates that the subgraph folding state has changed
    Folded { root: NodeIndex, subg: SubGraph },

    /// Indicates that the subgraph selection state has changed
    Selected { root: NodeIndex, subg: SubGraph },
}

impl ChangeSubgraph {
    pub(crate) fn change_folded(root: NodeIndex, subg: SubGraph) -> Self {
        Self::Folded { root, subg }
    }

    pub(crate) fn change_selected(root: NodeIndex, subg: SubGraph) -> Self {
        Self::Selected { root, subg }
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
    SubGraph(ChangeSubgraph),
}

impl Change {
    pub(crate) fn node(change: ChangeNode) -> Self {
        Self::Node(change)
    }

    pub(crate) fn edge(change: ChangeEdge) -> Self {
        Self::Edge(change)
    }
}
