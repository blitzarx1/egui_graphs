use petgraph::{stable_graph::EdgeIndex, stable_graph::NodeIndex};

use crate::{metadata::Metadata, selections::Selections, Node};

/// `StateComputed` is a utility struct for managing ephemerial state which is created and destroyed in one frame.
///
/// The struct stores the selected nodes, dragged node, and cached edges by nodes.
#[derive(Debug, Clone)]
pub struct StateComputed {
    pub dragged: Option<NodeIndex>,
    pub selections: Option<Selections>,
    nodes: Vec<StateComputedNode>,
    edges: Vec<StateComputedEdge>,
}

impl StateComputed {
    pub fn new(node_count: usize, edge_count: usize) -> Self {
        Self {
            dragged: None,
            selections: None,
            nodes: vec![Default::default(); node_count],
            edges: vec![Default::default(); edge_count],
        }
    }

    pub fn nodes_states(&self) -> &[StateComputedNode] {
        &self.nodes
    }

    pub fn edges_states(&self) -> &[StateComputedEdge] {
        &self.edges
    }

    pub fn node_state(&self, idx: NodeIndex) -> Option<&StateComputedNode> {
        self.nodes.get(idx.index())
    }

    pub fn node_state_mut(&mut self, idx: NodeIndex) -> Option<&mut StateComputedNode> {
        self.nodes.get_mut(idx.index())
    }

    pub fn edge_state(&self, idx: EdgeIndex) -> Option<&StateComputedEdge> {
        self.edges.get(idx.index())
    }

    pub fn edge_state_mut(&mut self, idx: EdgeIndex) -> Option<&mut StateComputedEdge> {
        self.edges.get_mut(idx.index())
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StateComputedNode {
    pub selected_child: bool,
    pub selected_parent: bool,
    radius: f32,
}

impl Default for StateComputedNode {
    fn default() -> Self {
        Self {
            selected_child: Default::default(),
            selected_parent: Default::default(),
            radius: 5.,
        }
    }
}

impl StateComputedNode {
    pub fn subselected(&self) -> bool {
        self.selected_child || self.selected_parent
    }

    pub fn radius(&self, meta: &Metadata) -> f32 {
        self.radius * meta.zoom
    }

    pub fn inc_radius(&mut self, inc: f32) {
        self.radius += inc;
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub struct StateComputedEdge {
    pub selected_child: bool,
    pub selected_parent: bool,
}

impl StateComputedEdge {
    pub fn subselected(&self) -> bool {
        self.selected_child || self.selected_parent
    }
}
