use egui::Pos2;

use petgraph::{
    stable_graph::{EdgeIndex, EdgeReference, NodeIndex},
    visit::{EdgeRef, IntoEdgeReferences, IntoNodeReferences},
    Direction, EdgeType,
};

use crate::{graph_view::Graph, metadata::Metadata, state_computed::StateComputed, Edge, Node};

/// Encapsulates graph access and traversal methods.
pub struct GraphWrapper<'a, N: Clone, E: Clone, Ty: EdgeType> {
    g: &'a mut Graph<N, E, Ty>,
}

impl<'a, N: Clone, E: Clone, Ty: EdgeType> GraphWrapper<'a, N, E, Ty> {
    pub fn new(g: &'a mut Graph<N, E, Ty>) -> Self {
        Self { g }
    }

    /// Iterates over all nodes and edges and calls the walker function.
    pub fn walk(
        &self,
        mut walker: impl FnMut(
            &Self,
            Option<&NodeIndex>,
            Option<&Node<N>>,
            Option<&EdgeIndex>,
            Option<&Edge<E>>,
        ),
    ) {
        self.nodes()
            .for_each(|(idx, n)| walker(self, Some(&idx), Some(n), None, None));
        self.edges()
            .for_each(|(idx, e)| walker(self, None, None, Some(&idx), Some(e)));
    }

    /// Finds node by position. Can be optimized by using a spatial index like quad-tree if needed.
    pub fn node_by_pos(
        &self,
        comp: &'a StateComputed,
        meta: &'a Metadata,
        pos: Pos2,
    ) -> Option<(NodeIndex, &Node<N>)> {
        // transform pos to graph coordinates
        let pos_in_graph = (pos - meta.pan).to_vec2() / meta.zoom;
        self.nodes().find(|(idx, n)| {
            let comp_node = comp.node_state(idx).unwrap();
            (n.location() - pos_in_graph).length() <= comp_node.radius
        })
    }

    ///Provides iterator over all nodes and their indices.
    pub fn nodes(&'a self) -> impl Iterator<Item = (NodeIndex, &Node<N>)> {
        self.g.node_references()
    }

    /// Provides iterator over all edges and their indices.
    pub fn edges(&'a self) -> impl Iterator<Item = (EdgeIndex, &Edge<E>)> {
        self.g.edge_references().map(|e| (e.id(), e.weight()))
    }

    pub fn node(&self, i: NodeIndex) -> Option<&Node<N>> {
        self.g.node_weight(i)
    }

    pub fn edge(&self, i: EdgeIndex) -> Option<&Edge<E>> {
        self.g.edge_weight(i)
    }

    pub fn edge_endpoints(&self, i: EdgeIndex) -> Option<(NodeIndex, NodeIndex)> {
        self.g.edge_endpoints(i)
    }

    pub fn node_mut(&mut self, i: NodeIndex) -> Option<&mut Node<N>> {
        self.g.node_weight_mut(i)
    }

    pub fn is_directed(&self) -> bool {
        self.g.is_directed()
    }

    pub fn edges_num(&self, idx: NodeIndex) -> usize {
        self.g.edges(idx).count()
    }

    pub fn edges_directed(
        &self,
        idx: NodeIndex,
        dir: Direction,
    ) -> impl Iterator<Item = EdgeReference<Edge<E>>> {
        self.g.edges_directed(idx, dir)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::Vec2;
    use petgraph::{stable_graph::StableGraph, Directed};

    fn create_test_graph() -> Graph<(), (), Directed> {
        let mut graph = StableGraph::<Node<()>, Edge<()>>::new();
        let a = graph.add_node(Node::new(Vec2::default(), ()));
        let b = graph.add_node(Node::new(Vec2::default(), ()));
        let c = graph.add_node(Node::new(Vec2::default(), ()));
        let d = graph.add_node(Node::new(Vec2::default(), ()));

        graph.add_edge(a, b, Edge::new(()));
        graph.add_edge(b, c, Edge::new(()));
        graph.add_edge(c, d, Edge::new(()));
        graph.add_edge(a, d, Edge::new(()));

        graph
    }

    #[test]
    fn test_walk() {
        let mut graph = create_test_graph();
        let graph_wrapped = GraphWrapper::new(&mut graph);
        let mut s = String::new();

        graph_wrapped.walk(|_g, n_idx, _n, e_idx, _e| {
            if let Some(_idx) = n_idx {
                s.push('n');
            };

            if let Some(_idx) = e_idx {
                s.push('e');
            };
        });

        //expecting n for every node and e for every edge in the graph
        assert_eq!(s, "nnnneeee".to_string());
    }
}
