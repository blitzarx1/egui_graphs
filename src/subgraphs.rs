use std::collections::HashMap;

use petgraph::{
    stable_graph::{EdgeIndex, NodeIndex},
    visit::EdgeRef,
    Direction, EdgeType, Graph,
};

use crate::graph_wrapper::GraphWrapper;

pub type Elements = (Vec<NodeIndex>, Vec<EdgeIndex>);

#[derive(Default, Debug, Clone)]
struct SubGraph {
    root: NodeIndex,
    g: Graph<NodeIndex, EdgeIndex>,
}

#[derive(Default, Debug, Clone)]
pub struct SubGraphs {
    data: HashMap<NodeIndex, SubGraph>,
}

impl SubGraph {
    fn from_root_and_depth<N: Clone, E: Clone, Ty: EdgeType>(
        g: &GraphWrapper<N, E, Ty>,
        root: NodeIndex,
        depth: i32,
    ) -> Self {
        let mut sub_g = Graph::<NodeIndex, EdgeIndex>::new();
        if depth == 0 {
            return SubGraph { root, g: sub_g };
        }

        let dir = match depth > 0 {
            true => petgraph::Direction::Outgoing,
            false => petgraph::Direction::Incoming,
        };

        collect_generations(g, &mut sub_g, root, depth.unsigned_abs() as usize, dir);

        Self { root, g: sub_g }
    }
}

impl SubGraphs {
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
        let g = &self.data.get(&root)?.g;

        if g.node_count() == 0 {
            return Some((vec![root], vec![]));
        }

        Some((
            g.node_weights().cloned().collect(),
            g.edge_weights().cloned().collect(),
        ))
    }

    pub fn add_subgraph<N: Clone, E: Clone, Ty: EdgeType>(
        &mut self,
        g: &GraphWrapper<N, E, Ty>,
        root: NodeIndex,
        depth: i32,
    ) {
        self.data
            .insert(root, SubGraph::from_root_and_depth(g, root, depth));
    }
}

fn collect_generations<N: Clone, E: Clone, Ty: EdgeType>(
    src: &GraphWrapper<N, E, Ty>,
    dst: &mut Graph<NodeIndex, EdgeIndex>,
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
            let s_idx = dst.add_node(*g_idx);
            src.edges_directed(*g_idx, dir).for_each(|edge| {
                let next = match dir {
                    Direction::Incoming => edge.source(),
                    Direction::Outgoing => edge.target(),
                };
                let next_idx = dst.add_node(next);
                dst.add_edge(s_idx, next_idx, edge.id());
                next_next_start.push(next);
            });
        });

        next_start = next_next_start;
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
    fn subgraphs_elements() {
        let g = &mut create_test_graph();
        let graph = GraphWrapper::new(g);
        let mut subgraphs = SubGraphs::default();

        // a->b, a->d
        subgraphs.add_subgraph(&graph, NodeIndex::new(0), 1);
        // b->c
        subgraphs.add_subgraph(&graph, NodeIndex::new(1), 1);

        let (nodes, edges) = subgraphs.elements();
        assert_eq!(nodes.len(), 4);
        assert_eq!(edges.len(), 3);
    }

    #[test]
    fn subgraphs_elements_by_root() {
        let g = &mut create_test_graph();
        let graph = GraphWrapper::new(g);
        let mut subgraphs = SubGraphs::default();

        subgraphs.add_subgraph(&graph, NodeIndex::new(0), 1);

        let (nodes, edges) = subgraphs.elements_by_root(NodeIndex::new(0)).unwrap();
        assert_eq!(nodes.len(), 3);
        assert_eq!(edges.len(), 2);
    }

    #[test]
    fn subgraphs_add_subgraph() {
        let g = &mut create_test_graph();
        let graph = GraphWrapper::new(g);
        let mut subgraphs = SubGraphs::default();

        subgraphs.add_subgraph(&graph, NodeIndex::new(0), 1);
        assert_eq!(subgraphs.data.len(), 1);

        subgraphs.add_subgraph(&graph, NodeIndex::new(1), 1);
        assert_eq!(subgraphs.data.len(), 2);
    }

    #[test]
    fn subgraphs_add_subgraph_zero_depth() {
        let g = &mut create_test_graph();
        let graph = GraphWrapper::new(g);
        let mut subgrpahs = SubGraphs::default();

        subgrpahs.add_subgraph(&graph, NodeIndex::new(0), 0);
        assert_eq!(subgrpahs.data.len(), 1);

        let subgrpaph = &subgrpahs.data.get(&NodeIndex::new(0)).unwrap().g;
        assert_eq!(subgrpaph.node_count(), 0);
        assert_eq!(subgrpaph.edge_count(), 0);
    }
}
