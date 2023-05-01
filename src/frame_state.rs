use std::collections::HashMap;

use petgraph::stable_graph::{EdgeIndex, NodeIndex, StableGraph};

use crate::{selections::Selections, Edge, Node};

pub(crate) type EdgesByNodes<E> = Vec<((usize, usize), Vec<(EdgeIndex, Edge<E>)>)>;

/// `FrameState` is a utility struct for managing ephemerial state which is created and destroyed in one frame.
///
/// The struct stores the selected nodes, dragged node, and cached edges by nodes.
#[derive(Debug, Clone)]
pub(crate) struct FrameState<E: Clone> {
    pub dragged: Option<NodeIndex>,
    pub selections: Option<Selections>,
    edges_by_nodes: Option<EdgesByNodes<E>>,
}

impl<E: Clone> Default for FrameState<E> {
    fn default() -> Self {
        Self {
            dragged: Default::default(),
            edges_by_nodes: Default::default(),
            selections: Default::default(),
        }
    }
}

impl<E: Clone> FrameState<E> {
    /// Helper method to get the edges by nodes. This is cached for performance.
    pub fn edges_by_nodes<N: Clone>(
        &mut self,
        g: &StableGraph<Node<N>, Edge<E>>,
    ) -> &EdgesByNodes<E> {
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
