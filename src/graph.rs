use egui::Pos2;
use petgraph::stable_graph::DefaultIx;
use petgraph::Directed;

use petgraph::graph::IndexType;
use petgraph::{
    stable_graph::{EdgeIndex, EdgeReference, NodeIndex, StableGraph},
    visit::{EdgeRef, IntoEdgeReferences, IntoNodeReferences},
    Direction, EdgeType,
};

use crate::draw::{DisplayEdge, DisplayNode};
use crate::{metadata::Metadata, transform, Edge, Node};

/// Graph type compatible with [`super::GraphView`].
#[derive(Debug, Clone)]
pub struct Graph<N: Clone, E: Clone, Ty: EdgeType = Directed, Ix: IndexType = DefaultIx> {
    pub g: StableGraph<Node<N, Ix>, Edge<E, Ix>, Ty, Ix>,
}

impl<N: Clone, E: Clone, Ty: EdgeType, Ix: IndexType> From<&StableGraph<N, E, Ty, Ix>>
    for Graph<N, E, Ty, Ix>
{
    fn from(value: &StableGraph<N, E, Ty, Ix>) -> Self {
        transform::to_graph(value)
    }
}

impl<N: Clone, E: Clone, Ty: EdgeType, Ix: IndexType> Graph<N, E, Ty, Ix> {
    pub fn new(g: StableGraph<Node<N, Ix>, Edge<E, Ix>, Ty, Ix>) -> Self {
        Self { g }
    }

    /// Finds node by position. Can be optimized by using a spatial index like quad-tree if needed.
    pub fn node_by_screen_pos<D: DisplayNode<N, E, Ty, Ix>>(
        &self,
        meta: &Metadata,
        screen_pos: Pos2,
    ) -> Option<(NodeIndex<Ix>, &Node<N, Ix>)> {
        let pos_in_graph = meta.screen_to_canvas_pos(screen_pos);
        for (idx, node) in self.nodes_iter() {
            if D::from(node.clone()).is_inside(self, pos_in_graph) {
                return Some((idx, node));
            }
        }
        None
    }

    /// Finds edge by position.
    pub fn edge_by_screen_pos<D: DisplayEdge<N, E, Ty, Ix>>(
        &self,
        meta: &Metadata,
        screen_pos: Pos2,
    ) -> Option<EdgeIndex<Ix>> {
        let pos_in_graph = meta.screen_to_canvas_pos(screen_pos);
        for (idx, e) in self.edges_iter() {
            if D::from(e.clone()).is_inside(self, pos_in_graph) {
                return Some(idx);
            }
        }

        None
    }

    pub fn g(&mut self) -> &mut StableGraph<Node<N, Ix>, Edge<E, Ix>, Ty, Ix> {
        &mut self.g
    }

    ///Provides iterator over all nodes and their indices.
    pub fn nodes_iter(&self) -> impl Iterator<Item = (NodeIndex<Ix>, &Node<N, Ix>)> {
        self.g.node_references()
    }

    /// Provides iterator over all edges and their indices.
    pub fn edges_iter(&self) -> impl Iterator<Item = (EdgeIndex<Ix>, &Edge<E, Ix>)> {
        self.g.edge_references().map(|e| (e.id(), e.weight()))
    }

    pub fn node(&self, i: NodeIndex<Ix>) -> Option<&Node<N, Ix>> {
        self.g.node_weight(i)
    }

    pub fn edge(&self, i: EdgeIndex<Ix>) -> Option<&Edge<E, Ix>> {
        self.g.edge_weight(i)
    }

    pub fn edge_endpoints(&self, i: EdgeIndex<Ix>) -> Option<(NodeIndex<Ix>, NodeIndex<Ix>)> {
        self.g.edge_endpoints(i)
    }

    pub fn node_mut(&mut self, i: NodeIndex<Ix>) -> Option<&mut Node<N, Ix>> {
        self.g.node_weight_mut(i)
    }

    pub fn edge_mut(&mut self, i: EdgeIndex<Ix>) -> Option<&mut Edge<E, Ix>> {
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
    ) -> impl Iterator<Item = EdgeReference<Edge<E, Ix>, Ix>> {
        self.g.edges_directed(idx, dir)
    }
}
