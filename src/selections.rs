use std::collections::HashMap;

use petgraph::{
    stable_graph::{EdgeIndex, NodeIndex},
    visit::EdgeRef,
    Direction, EdgeType, Graph,
};

use crate::graph_wrapper::GraphWrapper;

pub type Selection = Graph<NodeIndex, EdgeIndex>;
pub type Elements = (Vec<NodeIndex>, Vec<EdgeIndex>);

#[derive(Default, Debug, Clone)]
pub struct Selections {
    data: HashMap<NodeIndex, Selection>,
}

impl Selections {
    pub fn elements(&self) -> Elements {
        let mut nodes = vec![];
        let mut edges = vec![];

        for (root, _) in self.data.iter() {
            let (curr_nodes, curr_edges) = self.elements_by_root(*root).unwrap();
            nodes.extend(curr_nodes);
            edges.extend(curr_edges);
        }

        // remove duplicates
        nodes.sort();
        nodes.dedup();
        edges.sort();
        edges.dedup();

        (nodes, edges)
    }

    /// Walks the entire graph and collect node weights to `nodes`
    /// and edges weights to `edges`
    pub fn elements_by_root(&self, root: NodeIndex) -> Option<Elements> {
        let g = self.data.get(&root)?;

        if g.node_count() == 0 {
            return Some((vec![root], vec![]));
        }

        Some((
            g.node_weights().cloned().collect(),
            g.edge_weights().cloned().collect(),
        ))
    }

    pub fn add_selection<N: Clone, E: Clone, Ty: EdgeType>(
        &mut self,
        g: &GraphWrapper<N, E, Ty>,
        root: NodeIndex,
        depth: i32,
    ) {
        let mut selection_g = Graph::<NodeIndex, EdgeIndex>::new();
        if depth == 0 {
            self.data.insert(root, selection_g);
            return;
        }

        let dir = match depth > 0 {
            true => petgraph::Direction::Outgoing,
            false => petgraph::Direction::Incoming,
        };

        self.collect_generations(
            g,
            &mut selection_g,
            root,
            depth.unsigned_abs() as usize,
            dir,
        );

        self.data.insert(root, selection_g);
    }

    fn collect_generations<N: Clone, E: Clone, Ty: EdgeType>(
        &self,
        g: &GraphWrapper<N, E, Ty>,
        selection_g: &mut Graph<NodeIndex, EdgeIndex>,
        root: NodeIndex,
        n: usize,
        dir: Direction,
    ) {
        if n == 0 {
            return;
        }

        let mut depth = n;
        let mut next_start = vec![root];
        while depth > 0 {
            depth -= 1;

            let mut next_next_start = vec![];
            next_start.iter().for_each(|g_idx| {
                let s_idx = selection_g.add_node(*g_idx);
                g.edges_directed(*g_idx, dir).for_each(|edge| {
                    let next = match dir {
                        Direction::Incoming => edge.source(),
                        Direction::Outgoing => edge.target(),
                    };
                    let next_idx = selection_g.add_node(next);
                    selection_g.add_edge(s_idx, next_idx, edge.id());
                    next_next_start.push(next);
                });
            });

            next_start = next_next_start;
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{Edge, Node};

    use super::*;
    use egui::Vec2;
    use petgraph::stable_graph::StableGraph;

    fn create_test_graph() -> StableGraph<Node<()>, Edge<()>> {
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
    fn selections_elements() {
        let g = &mut create_test_graph();
        let graph = GraphWrapper::new(g);
        let mut selections = Selections::default();

        // a->b, a->d
        selections.add_selection(&graph, NodeIndex::new(0), 1);
        // b->c
        selections.add_selection(&graph, NodeIndex::new(1), 1);

        let (nodes, edges) = selections.elements();
        assert_eq!(nodes.len(), 4);
        assert_eq!(edges.len(), 3);
    }

    #[test]
    fn selections_elements_by_root() {
        let g = &mut create_test_graph();
        let graph = GraphWrapper::new(g);
        let mut selections = Selections::default();

        selections.add_selection(&graph, NodeIndex::new(0), 1);

        let (nodes, edges) = selections.elements_by_root(NodeIndex::new(0)).unwrap();
        assert_eq!(nodes.len(), 3);
        assert_eq!(edges.len(), 2);
    }

    #[test]
    fn selections_add_selection() {
        let g = &mut create_test_graph();
        let graph = GraphWrapper::new(g);
        let mut selections = Selections::default();

        selections.add_selection(&graph, NodeIndex::new(0), 1);
        assert_eq!(selections.data.len(), 1);

        selections.add_selection(&graph, NodeIndex::new(1), 1);
        assert_eq!(selections.data.len(), 2);
    }

    #[test]
    fn selections_add_selection_zero_depth() {
        let g = &mut create_test_graph();
        let graph = GraphWrapper::new(g);
        let mut selections = Selections::default();

        selections.add_selection(&graph, NodeIndex::new(0), 0);
        assert_eq!(selections.data.len(), 1);

        let selection_graph = selections.data.get(&NodeIndex::new(0)).unwrap();
        assert_eq!(selection_graph.node_count(), 0);
        assert_eq!(selection_graph.edge_count(), 0);
    }
}
