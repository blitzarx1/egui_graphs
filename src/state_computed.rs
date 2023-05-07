use std::collections::HashMap;

use petgraph::{stable_graph::EdgeIndex, stable_graph::NodeIndex};

use crate::{metadata::Metadata, subgraphs::Subgraphs};

/// `StateComputed` is a utility struct for managing ephemerial state which is created and destroyed in one frame.
///
/// The struct stores selections, dragged node and computed elements states.
#[derive(Default, Debug, Clone)]
pub struct StateComputed {
    pub dragged: Option<NodeIndex>,
    pub selections: Option<Subgraphs>,
    pub foldings: Option<Subgraphs>,
    pub nodes: HashMap<NodeIndex, StateComputedNode>,
    pub edges: HashMap<EdgeIndex, StateComputedEdge>,
}

impl StateComputed {
    pub fn node_state(&self, idx: &NodeIndex) -> Option<&StateComputedNode> {
        self.nodes.get(idx)
    }

    pub fn node_state_mut(&mut self, idx: &NodeIndex) -> Option<&mut StateComputedNode> {
        self.nodes.get_mut(idx)
    }

    pub fn edge_state(&self, idx: &EdgeIndex) -> Option<&StateComputedEdge> {
        self.edges.get(idx)
    }

    pub fn edge_state_mut(&mut self, idx: &EdgeIndex) -> Option<&mut StateComputedEdge> {
        self.edges.get_mut(idx)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StateComputedNode {
    pub selected_child: bool,
    pub selected_parent: bool,
    pub folding_root: Option<NodeIndex>,
    radius: f32,
}

impl Default for StateComputedNode {
    fn default() -> Self {
        Self {
            radius: 5.,

            selected_child: Default::default(),
            selected_parent: Default::default(),
            folding_root: Default::default(),
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

    pub fn is_folded(&self) -> bool {
        self.folding_root.is_some()
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub struct StateComputedEdge {
    pub selected_child: bool,
    pub selected_parent: bool,
    pub folded_ends: (u8, u8),
}

impl StateComputedEdge {
    pub fn subselected(&self) -> bool {
        self.selected_child || self.selected_parent
    }

    pub fn folded(&self) -> bool {
        self.folded_ends.0 != 0 || self.folded_ends.1 != 0
    }
}
