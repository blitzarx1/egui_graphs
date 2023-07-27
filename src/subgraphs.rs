use std::collections::{HashMap, HashSet};

use petgraph::{
    stable_graph::{EdgeIndex, NodeIndex},
    visit::EdgeRef,
    Direction, EdgeType, Graph,
};

use crate::graph_wrapper::GraphWrapper;

/// A subgraph of a graph. Node and edges are holding
/// references to the elements of the original graph.
pub type SubGraph = Graph<NodeIndex, EdgeIndex>;
pub type Elements = (Vec<NodeIndex>, Vec<EdgeIndex>);

/// A collection of [`SubGraph`]s. Each subgraph is identified by its root node.
#[derive(Default, Debug, Clone)]
pub struct SubGraphs {
    data: HashMap<NodeIndex, SubGraph>,
    /// Keeps references to the root node of the subgraph node.
    /// One node can be a part of multiple subgraphs.
    roots_by_node: HashMap<NodeIndex, Vec<NodeIndex>>,
}

impl SubGraphs {
    /// returns all subgraphs elements including its roots
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

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Walks the entire graph and collect node weights to `nodes`
    /// and edges weights to `edges`
    pub fn elements_by_root(&self, root: NodeIndex) -> Option<Elements> {
        let g = self.data.get(&root)?;

        if g.node_count() == 0 {
            return Some((vec![root], vec![]));
        }

        let mut nodes = g.node_weights().cloned().collect::<Vec<_>>();
        let mut edges = g.edge_weights().cloned().collect::<Vec<_>>();

        nodes.sort();
        nodes.dedup();
        edges.sort();
        edges.dedup();
        Some((nodes, edges))
    }

    /// Returns root nodes of the given node if it is a part of any subgraph.
    /// Is a way to check if the node is a part of any subgraph.
    pub fn roots_by_node(&self, idx: NodeIndex) -> Option<&Vec<NodeIndex>> {
        self.roots_by_node.get(&idx)
    }

    /// return all roots of the subgraphs
    pub fn roots(&self) -> Vec<NodeIndex> {
        self.data.keys().cloned().collect()
    }

    /// Adds a subgraph to the collection. The subgraph is created by walking the graph
    /// from the root node to the given depth.
    pub fn add_subgraph<N: Clone, E: Clone, Ty: EdgeType>(
        &mut self,
        g: &GraphWrapper<N, E, Ty>,
        root: NodeIndex,
        depth: i32,
    ) {
        let mut subgraph = Graph::<NodeIndex, EdgeIndex>::new();
        let dir = match depth > 0 {
            true => petgraph::Direction::Outgoing,
            false => petgraph::Direction::Incoming,
        };
        let steps = depth.unsigned_abs() as usize;
        self.collect_generations(g, &mut subgraph, root, steps, dir);

        self.data.insert(root, subgraph);
    }

    pub fn subgraphs(&self) -> impl Iterator<Item = (&NodeIndex, &SubGraph)> {
        self.data.iter()
    }

    fn add_node(&mut self, g: &mut SubGraph, root: NodeIndex, node: NodeIndex) -> NodeIndex {
        let idx = g.add_node(node);

        // do not add root to its roots
        if node == root {
            return idx;
        }

        if let Some(roots) = self.roots_by_node.get_mut(&node) {
            roots.push(root);
            roots.dedup();
            return idx;
        }

        self.roots_by_node.insert(node, vec![root]);
        idx
    }

    /// Bfs implementation which collects subgraph from `src_subgraph` starting from the `root` node
    /// travelling `steps` generations in the provided direction `dir`.
    fn collect_generations<N: Clone, E: Clone, Ty: EdgeType>(
        &mut self,
        src_subgraph: &GraphWrapper<N, E, Ty>,
        dst_subgraph: &mut Graph<NodeIndex, EdgeIndex>,
        root: NodeIndex,
        steps: usize,
        dir: Direction,
    ) {
        let mut visited = HashSet::new();
        let mut steps_left = steps;
        let mut nodes = vec![root];
        while steps_left > 0 && !nodes.is_empty() {
            steps_left -= 1;

            let mut next_nodes = vec![];
            nodes.iter().for_each(|src_start_idx| {
                let dst_start_idx = self.add_node(dst_subgraph, root, *src_start_idx);
                src_subgraph
                    .edges_directed(*src_start_idx, dir)
                    .for_each(|edge| {
                        let src_next_idx = match dir {
                            Direction::Incoming => edge.source(),
                            Direction::Outgoing => edge.target(),
                        };

                        let src_edge_idx = edge.id();

                        if !visited.insert((*src_start_idx, src_next_idx, src_edge_idx)) {
                            return;
                        }

                        let dst_next_idx = self.add_node(dst_subgraph, root, src_next_idx);
                        dst_subgraph.add_edge(dst_start_idx, dst_next_idx, src_edge_idx);
                        next_nodes.push(src_next_idx);
                    });
            });

            nodes = next_nodes;
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

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
    fn subgraphs_add_looped_subgraph() {
        let g = &mut create_test_graph();
        let a_idx = NodeIndex::new(1);
        g.add_edge(a_idx, a_idx, Edge::new(()));

        let graph = GraphWrapper::new(g);
        let mut subgraphs = SubGraphs::default();

        subgraphs.add_subgraph(&graph, NodeIndex::new(1), i32::MAX);
        assert_eq!(subgraphs.data.len(), 1);
    }

    #[test]
    fn subgraphs_add_subgraph_zero_depth() {
        let g = &mut create_test_graph();
        let graph = GraphWrapper::new(g);
        let mut subgraphs = SubGraphs::default();

        subgraphs.add_subgraph(&graph, NodeIndex::new(0), 0);
        assert_eq!(subgraphs.data.len(), 1);

        let subgraph = subgraphs.data.get(&NodeIndex::new(0)).unwrap();
        assert_eq!(subgraph.node_count(), 0);
        assert_eq!(subgraph.edge_count(), 0);
    }

    #[test]
    fn subgraphs_roots_by_node() {
        let g = &mut create_test_graph();
        let graph = GraphWrapper::new(g);
        let mut subgraphs = SubGraphs::default();

        // a->b, a->d
        subgraphs.add_subgraph(&graph, NodeIndex::new(0), 1);
        // b->c
        subgraphs.add_subgraph(&graph, NodeIndex::new(1), 1);

        // Check roots for node 1 (b)
        let roots = subgraphs.roots_by_node(NodeIndex::new(1)).unwrap();
        assert_eq!(roots.len(), 1);
        assert_eq!(roots[0], NodeIndex::new(0));

        // Check roots for node 2 (c)
        let roots = subgraphs.roots_by_node(NodeIndex::new(2)).unwrap();
        assert_eq!(roots.len(), 1);
        assert_eq!(roots[0], NodeIndex::new(1));
    }

    #[test]
    fn subgraphs_roots_and_roots_by_node() {
        // Check that roots and roots_by_node are the same

        let g = &mut create_test_graph();
        let graph = GraphWrapper::new(g);
        let mut subgraphs = SubGraphs::default();

        // a->b, a->d
        subgraphs.add_subgraph(&graph, NodeIndex::new(0), 1);
        // b->c
        subgraphs.add_subgraph(&graph, NodeIndex::new(1), 1);

        let all_roots_from_roots_by_node = subgraphs
            .roots_by_node
            .values()
            .cloned()
            .reduce(|acc, el| acc.into_iter().chain(el.into_iter()).collect())
            .unwrap();
        let all_roots_from_roots_by_node_deduped =
            all_roots_from_roots_by_node.iter().collect::<HashSet<_>>();

        assert_eq!(
            subgraphs.roots().len(),
            all_roots_from_roots_by_node_deduped.len()
        );
        subgraphs.roots().iter().for_each(|root| {
            assert!(all_roots_from_roots_by_node_deduped.contains(root));
        });
    }
}
