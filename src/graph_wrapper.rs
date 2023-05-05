use egui::Pos2;
use petgraph::{
    stable_graph::{EdgeIndex, EdgeReference, NodeIndex, StableGraph},
    visit::{IntoEdgeReferences, IntoNodeReferences},
    Direction::{self, Incoming, Outgoing},
};

use crate::{
    metadata::Metadata,
    state_computed::{StateComputed, StateComputedNode},
    Edge, Node,
};

/// Encapsulates graph access and traversal methods.
pub struct GraphWrapper<'a, N: Clone, E: Clone> {
    g: &'a mut StableGraph<Node<N>, Edge<E>>,
}

impl<'a, N: Clone, E: Clone> GraphWrapper<'a, N, E> {
    pub fn new(g: &'a mut StableGraph<Node<N>, Edge<E>>) -> Self {
        Self { g }
    }

    pub fn node_by_pos(
        &self,
        comp: &'a StateComputed,
        meta: &'a Metadata,
        pos: Pos2,
    ) -> Option<(NodeIndex, &Node<N>, &StateComputedNode)> {
        // transform pos to graph coordinates
        let pos_in_graph = (pos - meta.pan).to_vec2() / meta.zoom;
        self.nodes_with_context(comp)
            .find(|(_, n, comp)| (n.location - pos_in_graph).length() <= comp.radius(meta))
    }

    pub fn nodes_with_context(
        &'a self,
        comp: &'a StateComputed,
    ) -> impl Iterator<Item = (NodeIndex, &Node<N>, &StateComputedNode)> {
        self.g
            .node_references()
            .map(|(i, n)| (i, n, comp.node_state(&i).unwrap()))
    }

    pub fn nodes(&'a self) -> impl Iterator<Item = (NodeIndex, &Node<N>)> {
        self.g.node_references()
    }

    pub fn edges(&'a self) -> impl Iterator<Item = EdgeReference<Edge<E>>> {
        self.g.edge_references()
    }

    pub fn node(&self, i: NodeIndex) -> Option<&Node<N>> {
        self.g.node_weight(i)
    }

    pub fn edge(&self, i: EdgeIndex) -> Option<&Edge<E>> {
        self.g.edge_weight(i)
    }

    pub fn node_mut(&mut self, i: NodeIndex) -> Option<&mut Node<N>> {
        self.g.node_weight_mut(i)
    }

    pub fn is_directed(&self) -> bool {
        self.g.is_directed()
    }

    pub fn edges_num(&self, idx: NodeIndex) -> usize {
        if self.is_directed() {
            self.g
                .edges_directed(idx, Outgoing)
                .chain(self.g.edges_directed(idx, Incoming))
                .count()
        } else {
            self.g.edges(idx).count()
        }
    }

    pub fn node_count(&self) -> usize {
        self.g.node_count()
    }

    pub fn edge_count(&self) -> usize {
        self.g.edge_count()
    }

    pub fn edges_directed(
        &self,
        idx: NodeIndex,
        dir: Direction,
    ) -> impl Iterator<Item = EdgeReference<Edge<E>>> {
        self.g.edges_directed(idx, dir)
    }
}
