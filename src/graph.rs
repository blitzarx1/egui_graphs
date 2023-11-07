use egui::Pos2;
use petgraph::graph::IndexType;
use petgraph::Directed;

use petgraph::stable_graph::DefaultIx;
use petgraph::{
    stable_graph::{EdgeIndex, EdgeReference, NodeIndex, StableGraph},
    visit::{EdgeRef, IntoEdgeReferences, IntoNodeReferences},
    Direction, EdgeType,
};

use crate::{metadata::Metadata, transform, Edge, Node, SettingsStyle};

/// Graph type compatible with [`super::GraphView`].
#[derive(Debug, Clone)]
pub struct Graph<N: Clone, E: Clone, Ty: EdgeType = Directed, Ix: IndexType = DefaultIx> {
    pub g: StableGraph<Node<N>, Edge<E>, Ty, Ix>,
}

impl<N: Clone, E: Clone, Ty: EdgeType, Ix: IndexType> From<&StableGraph<N, E, Ty, Ix>>
    for Graph<N, E, Ty, Ix>
{
    fn from(value: &StableGraph<N, E, Ty, Ix>) -> Self {
        transform::to_graph(value)
    }
}

impl<'a, N: Clone, E: Clone, Ty: EdgeType, Ix: IndexType> Graph<N, E, Ty, Ix> {
    pub fn new(g: StableGraph<Node<N>, Edge<E>, Ty, Ix>) -> Self {
        Self { g }
    }

    /// Finds node by position. Can be optimized by using a spatial index like quad-tree if needed.
    pub fn node_by_screen_pos(
        &self,
        meta: &'a Metadata,
        style: &'a SettingsStyle,
        screen_pos: Pos2,
    ) -> Option<(NodeIndex<Ix>, &Node<N>)> {
        let pos_in_graph = (screen_pos.to_vec2() - meta.pan) / meta.zoom;
        self.nodes_iter().find(|(_, n)| {
            let dist_to_node = (n.location() - pos_in_graph).length();
            dist_to_node <= n.screen_radius(meta, style) / meta.zoom
        })
    }

    pub fn g(&mut self) -> &mut StableGraph<Node<N>, Edge<E>, Ty, Ix> {
        &mut self.g
    }

    ///Provides iterator over all nodes and their indices.
    pub fn nodes_iter(&'a self) -> impl Iterator<Item = (NodeIndex<Ix>, &Node<N>)> {
        self.g.node_references()
    }

    /// Provides iterator over all edges and their indices.
    pub fn edges_iter(&'a self) -> impl Iterator<Item = (EdgeIndex<Ix>, &Edge<E>)> {
        self.g.edge_references().map(|e| (e.id(), e.weight()))
    }

    pub fn node(&self, i: NodeIndex<Ix>) -> Option<&Node<N>> {
        self.g.node_weight(i)
    }

    pub fn edge(&self, i: EdgeIndex<Ix>) -> Option<&Edge<E>> {
        self.g.edge_weight(i)
    }

    pub fn edge_endpoints(&self, i: EdgeIndex<Ix>) -> Option<(NodeIndex<Ix>, NodeIndex<Ix>)> {
        self.g.edge_endpoints(i)
    }

    pub fn node_mut(&mut self, i: NodeIndex<Ix>) -> Option<&mut Node<N>> {
        self.g.node_weight_mut(i)
    }

    pub fn edge_mut(&mut self, i: EdgeIndex<Ix>) -> Option<&mut Edge<E>> {
        self.g.edge_weight_mut(i)
    }

    pub fn is_directed(&self) -> bool {
        self.g.is_directed()
    }

    pub fn edges_num(&self, idx: NodeIndex<Ix>) -> usize {
        self.g.edges(idx).count()
    }

    pub fn edges_directed(
        &self,
        idx: NodeIndex<Ix>,
        dir: Direction,
    ) -> impl Iterator<Item = EdgeReference<Edge<E>, Ix>> {
        self.g.edges_directed(idx, dir)
    }
}
