use egui::epaint::ahash::HashMap;
use petgraph::{
    stable_graph::{EdgeIndex, NodeIndex, StableGraph},
    visit::EdgeRef,
    Direction, Graph,
};

use crate::{Edge, Node};

pub(crate) type Selection = Graph<NodeIndex, EdgeIndex>;
pub(crate) type Elements = (Vec<NodeIndex>, Vec<EdgeIndex>);

#[derive(Default, Debug, Clone)]
pub(crate) struct Selections {
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

    pub fn add_selection<N: Clone, E: Clone>(
        &mut self,
        g: &StableGraph<Node<N>, Edge<E>>,
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

    fn collect_generations<N: Clone, E: Clone>(
        &self,
        g: &StableGraph<Node<N>, Edge<E>>,
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
