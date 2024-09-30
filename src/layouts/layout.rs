use petgraph::stable_graph::{IndexType, NodeIndex};

use crate::Graph;

pub trait Layout {
    fn next(&mut self, g: &mut Graph);
    fn next_for_node<Ix: IndexType>(&mut self, g: &mut Graph, idx: NodeIndex<Ix>);
}
