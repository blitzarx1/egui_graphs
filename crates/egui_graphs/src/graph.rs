use egui::{Pos2, Rect};
use petgraph::stable_graph::DefaultIx;
use petgraph::Directed;

use petgraph::graph::IndexType;
use petgraph::{
    stable_graph::{EdgeIndex, EdgeReference, NodeIndex, StableGraph},
    visit::{EdgeRef, IntoEdgeReferences, IntoNodeReferences},
    Direction, EdgeType,
};
use serde::{Deserialize, Serialize};

use crate::draw::{DisplayEdge, DisplayNode};
use crate::{
    default_edge_transform, default_node_transform, to_graph, DefaultEdgeShape, DefaultNodeShape,
};
use crate::{metadata::MetadataFrame, Edge, Node};

type StableGraphType<N, E, Ty, Ix, Dn, De> =
    StableGraph<Node<N, E, Ty, Ix, Dn>, Edge<N, E, Ty, Ix, Dn, De>, Ty, Ix>;

/// Wrapper around [`petgraph::stable_graph::StableGraph`] compatible with [`super::GraphView`].
/// It is used to store graph data and provide access to it.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Graph<
    N = (),
    E = (),
    Ty = Directed,
    Ix = DefaultIx,
    Dn = DefaultNodeShape,
    De = DefaultEdgeShape,
> where
    N: Clone,
    E: Clone,
    Ty: EdgeType,
    Ix: IndexType,
    Dn: DisplayNode<N, E, Ty, Ix>,
    De: DisplayEdge<N, E, Ty, Ix, Dn>,
{
    g: StableGraphType<N, E, Ty, Ix, Dn, De>,

    selected_nodes: Vec<NodeIndex<Ix>>,
    selected_edges: Vec<EdgeIndex<Ix>>,
    dragged_node: Option<NodeIndex<Ix>>,
    hovered_node: Option<NodeIndex<Ix>>,

    bounds: Rect,
}

impl<N, E, Ty, Ix, Dn, De> From<&StableGraph<N, E, Ty, Ix>> for Graph<N, E, Ty, Ix, Dn, De>
where
    N: Clone,
    E: Clone,
    Ty: EdgeType,
    Ix: IndexType,
    Dn: DisplayNode<N, E, Ty, Ix>,
    De: DisplayEdge<N, E, Ty, Ix, Dn>,
{
    fn from(g: &StableGraph<N, E, Ty, Ix>) -> Self {
        to_graph(g)
    }
}

impl<N, E, Ty, Ix, Dn, De> Graph<N, E, Ty, Ix, Dn, De>
where
    N: Clone,
    E: Clone,
    Ty: EdgeType,
    Ix: IndexType,
    Dn: DisplayNode<N, E, Ty, Ix>,
    De: DisplayEdge<N, E, Ty, Ix, Dn>,
{
    pub fn new(g: StableGraphType<N, E, Ty, Ix, Dn, De>) -> Self {
        Self {
            g,
            selected_nodes: Vec::default(),
            selected_edges: Vec::default(),
            dragged_node: Option::default(),
            hovered_node: Option::default(),
            bounds: Rect::from_min_max(Pos2::ZERO, Pos2::ZERO),
        }
    }

    /// Finds node by position. Can be optimized by using a spatial index like quad-tree if needed.
    pub fn node_by_screen_pos(
        &self,
        meta: &MetadataFrame,
        screen_pos: Pos2,
    ) -> Option<NodeIndex<Ix>> {
        let pos_in_graph = meta.screen_to_canvas_pos(screen_pos);
        for (idx, node) in self.nodes_iter() {
            let display = node.display();
            if display.is_inside(pos_in_graph) {
                return Some(idx);
            }
        }
        None
    }

    /// Finds edge by position.
    #[allow(clippy::missing_panics_doc)] // TODO: add panics doc
    pub fn edge_by_screen_pos(
        &self,
        meta: &MetadataFrame,
        screen_pos: Pos2,
    ) -> Option<EdgeIndex<Ix>> {
        let pos_in_graph = meta.screen_to_canvas_pos(screen_pos);
        for (idx, e) in self.edges_iter() {
            let Some((idx_start, idx_end)) = self.g.edge_endpoints(e.id()) else {
                continue;
            };
            let start = self.g.node_weight(idx_start).unwrap();
            let end = self.g.node_weight(idx_end).unwrap();
            if e.display().is_inside(start, end, pos_in_graph) {
                return Some(idx);
            }
        }

        None
    }

    pub fn g_mut(&mut self) -> &mut StableGraphType<N, E, Ty, Ix, Dn, De> {
        &mut self.g
    }

    pub fn g(&self) -> &StableGraphType<N, E, Ty, Ix, Dn, De> {
        &self.g
    }

    /// Adds node to graph setting default location and default label values
    #[allow(clippy::missing_panics_doc)] // TODO: add panics doc
    pub fn add_node(&mut self, payload: N) -> NodeIndex<Ix> {
        self.add_node_custom(payload, default_node_transform)
    }

    #[allow(clippy::missing_panics_doc)] // TODO: add panics doc
    pub fn add_node_custom(
        &mut self,
        payload: N,
        node_transform: impl FnOnce(&mut Node<N, E, Ty, Ix, Dn>),
    ) -> NodeIndex<Ix> {
        let node = Node::new(payload);

        let idx = self.g.add_node(node);
        let graph_node = self.g.node_weight_mut(idx).unwrap();

        graph_node.set_id(idx);

        node_transform(graph_node);

        idx
    }

    /// Adds node to graph setting custom location and default label value
    #[allow(clippy::missing_panics_doc)] // TODO: add panics doc
    pub fn add_node_with_location(&mut self, payload: N, location: Pos2) -> NodeIndex<Ix> {
        self.add_node_custom(payload, |n: &mut Node<N, E, Ty, Ix, Dn>| {
            n.set_location(location);
        })
    }

    /// Adds node to graph setting default location and custom label value
    pub fn add_node_with_label(&mut self, payload: N, label: String) -> NodeIndex<Ix> {
        self.add_node_custom(payload, |n: &mut Node<N, E, Ty, Ix, Dn>| {
            n.set_label(label);
        })
    }

    /// Adds node to graph setting custom location and custom label value
    #[allow(clippy::missing_panics_doc)] // TODO: add panics doc
    pub fn add_node_with_label_and_location(
        &mut self,
        payload: N,
        label: String,
        location: Pos2,
    ) -> NodeIndex<Ix> {
        self.add_node_custom(payload, |n: &mut Node<N, E, Ty, Ix, Dn>| {
            n.set_location(location);
            n.set_label(label);
        })
    }

    /// Removes node by index. Returns removed node and None if it does not exist.
    pub fn remove_node(&mut self, idx: NodeIndex<Ix>) -> Option<Node<N, E, Ty, Ix, Dn>> {
        // before removing nodes we need to remove all edges connected to it
        let neighbors = self.g.neighbors_undirected(idx).collect::<Vec<_>>();
        for n in &neighbors {
            self.remove_edges_between(idx, *n);
            self.remove_edges_between(*n, idx);
        }

        self.g.remove_node(idx)
    }

    /// Removes all edges between start and end node. Returns removed edges count.
    #[allow(clippy::missing_panics_doc)] // TODO: add panics doc
    pub fn remove_edges_between(&mut self, start: NodeIndex<Ix>, end: NodeIndex<Ix>) -> usize {
        let idxs = self
            .g
            .edges_connecting(start, end)
            .map(|e| e.id())
            .collect::<Vec<_>>();
        if idxs.is_empty() {
            return 0;
        }

        let mut removed = 0;
        for e in &idxs {
            self.g.remove_edge(*e).unwrap();
            removed += 1;
        }

        removed
    }

    /// Adds edge between start and end node with default label.
    #[allow(clippy::missing_panics_doc)] // TODO: add panics doc
    pub fn add_edge(
        &mut self,
        start: NodeIndex<Ix>,
        end: NodeIndex<Ix>,
        payload: E,
    ) -> EdgeIndex<Ix> {
        self.add_edge_custom(start, end, payload, default_edge_transform)
    }

    /// Adds edge between start and end node with custom label setting correct order.
    #[allow(clippy::missing_panics_doc)] // TODO: add panics doc
    pub fn add_edge_with_label(
        &mut self,
        start: NodeIndex<Ix>,
        end: NodeIndex<Ix>,
        payload: E,
        label: String,
    ) -> EdgeIndex<Ix> {
        self.add_edge_custom(start, end, payload, |e: &mut Edge<N, E, Ty, Ix, Dn, De>| {
            e.set_label(label);
        })
    }

    /// Dynamically recalculates and packs the visual order of edges between two nodes.
    fn update_edge_orders(&mut self, start: NodeIndex<Ix>, end: NodeIndex<Ix>) {
        let mut ab_edges: Vec<_> = self
            .g
            .edges_connecting(start, end)
            .map(|e| e.id())
            .collect();
        let mut ba_edges: Vec<_> = self
            .g
            .edges_connecting(end, start)
            .map(|e| e.id())
            .collect();

        // Deduplicate: naturally empties ba_edges for undirected graphs and self-loops
        ba_edges.retain(|id| !ab_edges.contains(id));

        // Sort by current order to preserve the visual sequence of existing edges
        ab_edges.sort_by_key(|&id| self.g.edge_weight(id).unwrap().order());
        ba_edges.sort_by_key(|&id| self.g.edge_weight(id).unwrap().order());

        // Reserve order 0 (straight line) if opposing traffic exists
        let offset = if !ab_edges.is_empty() && !ba_edges.is_empty() {
            1
        } else {
            0
        };

        for (i, id) in ab_edges.into_iter().enumerate() {
            self.g.edge_weight_mut(id).unwrap().set_order(i + offset);
        }
        for (i, id) in ba_edges.into_iter().enumerate() {
            self.g.edge_weight_mut(id).unwrap().set_order(i + offset);
        }
    }

    #[allow(clippy::missing_panics_doc)] // TODO: add panics doc
    pub fn add_edge_custom(
        &mut self,
        start: NodeIndex<Ix>,
        end: NodeIndex<Ix>,
        payload: E,
        edge_transform: impl FnOnce(&mut Edge<N, E, Ty, Ix, Dn, De>),
    ) -> EdgeIndex<Ix> {
        let idx = self.g.add_edge(start, end, Edge::new(payload));
        let e = self.g.edge_weight_mut(idx).unwrap();

        e.set_id(idx);
        e.set_order(usize::MAX);

        edge_transform(e);

        self.update_edge_orders(start, end);

        idx
    }

    /// Removes edge by index and updates order of the siblings.
    /// Returns removed edge and None if it does not exist.
    pub fn remove_edge(&mut self, idx: EdgeIndex<Ix>) -> Option<Edge<N, E, Ty, Ix, Dn, De>> {
        let (start, end) = self.g.edge_endpoints(idx)?;
        let payload = self.g.remove_edge(idx)?;

        self.update_edge_orders(start, end);

        Some(payload)
    }

    /// Returns iterator over all edges connecting start and end node.
    #[allow(clippy::type_complexity)]
    pub fn edges_connecting(
        &self,
        start: NodeIndex<Ix>,
        end: NodeIndex<Ix>,
    ) -> impl Iterator<Item = (EdgeIndex<Ix>, &Edge<N, E, Ty, Ix, Dn, De>)> {
        self.g
            .edges_connecting(start, end)
            .map(|e| (e.id(), e.weight()))
    }

    /// Provides iterator over all nodes and their indices.
    pub fn nodes_iter(&self) -> impl Iterator<Item = (NodeIndex<Ix>, &Node<N, E, Ty, Ix, Dn>)> {
        self.g.node_references()
    }

    /// Provides iterator over all edges and their indices.
    #[allow(clippy::type_complexity)]
    pub fn edges_iter(&self) -> impl Iterator<Item = (EdgeIndex<Ix>, &Edge<N, E, Ty, Ix, Dn, De>)> {
        self.g.edge_references().map(|e| (e.id(), e.weight()))
    }

    pub fn node(&self, i: NodeIndex<Ix>) -> Option<&Node<N, E, Ty, Ix, Dn>> {
        self.g.node_weight(i)
    }

    pub fn edge(&self, i: EdgeIndex<Ix>) -> Option<&Edge<N, E, Ty, Ix, Dn, De>> {
        self.g.edge_weight(i)
    }

    pub fn edge_endpoints(&self, i: EdgeIndex<Ix>) -> Option<(NodeIndex<Ix>, NodeIndex<Ix>)> {
        self.g.edge_endpoints(i)
    }

    pub fn node_mut(&mut self, i: NodeIndex<Ix>) -> Option<&mut Node<N, E, Ty, Ix, Dn>> {
        self.g.node_weight_mut(i)
    }

    pub fn edge_mut(&mut self, i: EdgeIndex<Ix>) -> Option<&mut Edge<N, E, Ty, Ix, Dn, De>> {
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
    ) -> impl Iterator<Item = EdgeReference<'_, Edge<N, E, Ty, Ix, Dn, De>, Ix>> {
        self.g.edges_directed(idx, dir)
    }

    pub fn selected_nodes(&self) -> &[NodeIndex<Ix>] {
        &self.selected_nodes
    }

    pub fn set_selected_nodes(&mut self, nodes: Vec<NodeIndex<Ix>>) {
        self.selected_nodes = nodes;
    }

    pub fn selected_edges(&self) -> &[EdgeIndex<Ix>] {
        &self.selected_edges
    }

    pub fn set_selected_edges(&mut self, edges: Vec<EdgeIndex<Ix>>) {
        self.selected_edges = edges;
    }

    pub fn dragged_node(&self) -> Option<NodeIndex<Ix>> {
        self.dragged_node
    }

    pub fn set_dragged_node(&mut self, node: Option<NodeIndex<Ix>>) {
        self.dragged_node = node;
    }

    pub fn hovered_node(&self) -> Option<NodeIndex<Ix>> {
        self.hovered_node
    }

    pub fn set_hovered_node(&mut self, node: Option<NodeIndex<Ix>>) {
        self.hovered_node = node;
    }

    pub fn edge_count(&self) -> usize {
        self.g.edge_count()
    }

    pub fn node_count(&self) -> usize {
        self.g.node_count()
    }

    pub fn set_bounds(&mut self, bounds: Rect) {
        self.bounds = bounds;
    }

    pub fn bounds(&self) -> Rect {
        self.bounds
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use petgraph::stable_graph::StableGraph;

    #[test]
    fn edge_orders_pack_densely_with_bidirectional_offset() {
        // Directed graph with default display types
        let mut sg: StableGraph<(), ()> = StableGraph::default();
        let a = sg.add_node(());
        let b = sg.add_node(());
        let mut g: Graph<(), (), Directed> =
            Graph::new(sg.map(|_, ()| crate::Node::new(()), |_, ()| crate::Edge::new(())));

        // 1. Add opposite-direction edges
        let e1 = g.add_edge(a, b, ());
        let e2 = g.add_edge(b, a, ());

        assert_eq!(
            g.edge(e1).unwrap().order(),
            1,
            "A->B should be offset to 1 due to bidirectional traffic"
        );
        assert_eq!(
            g.edge(e2).unwrap().order(),
            1,
            "B->A should be offset to 1 due to bidirectional traffic"
        );

        // 2. Add a second A->B edge
        let e3 = g.add_edge(a, b, ());
        assert_eq!(
            g.edge(e3).unwrap().order(),
            2,
            "Second A->B edge should stack sequentially at order 2"
        );
        // Ensure e1 remained at 1
        assert_eq!(g.edge(e1).unwrap().order(), 1);

        // Ensure e2 remained at 1
        assert_eq!(
            g.edge(e2).unwrap().order(),
            1,
            "B->A order should not have changed"
        );

        // 3. Add a third A->B edge
        let e4 = g.add_edge(a, b, ());
        assert_eq!(
            g.edge(e4).unwrap().order(),
            3,
            "Third A->B edge should stack sequentially at order 3"
        );
    }
}
