use std::collections::HashMap;

use petgraph::stable_graph::{EdgeIndex, NodeIndex, StableGraph};

use crate::{Edge, Node};

/// `FrameState` is a utility struct for managing ephemerial state which is created and destroyed in one frame.
///
/// The struct stores the selected nodes, dragged node, and cached edges by nodes.
#[derive(Debug, Clone)]
pub struct FrameState<E: Clone> {
    pub selected_nodes: Vec<NodeIndex>,
    pub selected_edges: Vec<EdgeIndex>,
    pub dragged: Option<NodeIndex>,
    edges_by_nodes: Option<Vec<((usize, usize), Vec<(EdgeIndex, Edge<E>)>)>>,
}

impl<E: Clone> Default for FrameState<E> {
    fn default() -> Self {
        Self {
            selected_nodes: Default::default(),
            selected_edges: Default::default(),
            dragged: None,
            edges_by_nodes: None,
        }
    }
}

impl<E: Clone> FrameState<E> {
    /// Helper method to get the edges by nodes. This is cached for performance.
    pub fn edges_by_nodes<N: Clone>(
        &mut self,
        g: &StableGraph<Node<N>, Edge<E>>,
    ) -> &Vec<((usize, usize), Vec<(EdgeIndex, Edge<E>)>)> {
        if self.edges_by_nodes.is_some() {
            return self.edges_by_nodes.as_ref().unwrap();
        }

        let mut edge_map: HashMap<(usize, usize), Vec<(EdgeIndex, Edge<E>)>> = HashMap::new();

        for edge_idx in g.edge_indices() {
            let (source_idx, target_idx) = g.edge_endpoints(edge_idx).unwrap();
            let source = source_idx.index();
            let target = target_idx.index();
            let edge = g.edge_weight(edge_idx).unwrap().clone();

            edge_map
                .entry((source, target))
                .or_insert_with(Vec::new)
                .push((edge_idx, edge));
        }

        let res = edge_map
            .iter()
            .map(|entry| (*entry.0, entry.1.clone()))
            .collect::<Vec<_>>();

        self.edges_by_nodes = Some(res);

        self.edges_by_nodes.as_ref().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::Vec2;
    use petgraph::stable_graph::StableGraph;

    // Helper function to create a test StableGraph
    fn create_test_graph() -> StableGraph<Node<()>, Edge<usize>> {
        let mut graph = StableGraph::<Node<()>, Edge<usize>>::new();
        let n0 = graph.add_node(Node::new(Vec2::default(), ()));
        let n1 = graph.add_node(Node::new(Vec2::default(), ()));
        let n2 = graph.add_node(Node::new(Vec2::default(), ()));

        graph.add_edge(n0, n1, Edge::new(1));
        graph.add_edge(n0, n2, Edge::new(2));
        graph.add_edge(n1, n2, Edge::new(3));

        graph
    }

    #[test]
    fn test_frame_state_default() {
        let frame_state: FrameState<usize> = FrameState::default();
        assert_eq!(frame_state.selected_nodes.len(), 0);
        assert!(frame_state.dragged.is_none());
        assert!(frame_state.edges_by_nodes.is_none());
    }

    #[test]
    fn test_edges_by_nodes() {
        let graph = create_test_graph();
        let mut frame_state = FrameState::<usize>::default();
        let edges_by_nodes = frame_state.edges_by_nodes(&graph);

        // Verify the size of the output vector
        assert_eq!(edges_by_nodes.len(), 3);

        // Verify that edges_by_nodes contains the correct edges
        let mut found_edges = HashMap::new();
        for ((source, target), edges) in edges_by_nodes {
            found_edges.insert((*source, *target), edges);
        }

        for edge_idx in graph.edge_indices() {
            let (source_idx, target_idx) = graph.edge_endpoints(edge_idx).unwrap();
            let source = source_idx.index();
            let target = target_idx.index();
            let edge = graph.edge_weight(edge_idx).unwrap();

            assert!(found_edges
                .get(&(source, target))
                .unwrap()
                .iter()
                .any(|(idx, e)| idx == &edge_idx && e == edge));
        }
    }
}
