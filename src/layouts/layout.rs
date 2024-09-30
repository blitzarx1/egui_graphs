use petgraph::{
    stable_graph::{IndexType, NodeIndex},
    EdgeType,
};

use crate::{DisplayEdge, DisplayNode, Graph};

pub trait Layout: Default {
    fn next<
        N: Clone,
        E: Clone,
        Ty: EdgeType,
        Ix: IndexType,
        Dn: DisplayNode<N, E, Ty, Ix>,
        De: DisplayEdge<N, E, Ty, Ix, Dn>,
        L: Layout,
    >(
        &mut self,
        g: &mut Graph<N, E, Ty, Ix, Dn, De, L>,
    );
    fn next_for_node<
        N: Clone,
        E: Clone,
        Ty: EdgeType,
        Ix: IndexType,
        Dn: DisplayNode<N, E, Ty, Ix>,
        De: DisplayEdge<N, E, Ty, Ix, Dn>,
        L: Layout,
    >(
        &mut self,
        g: &mut Graph<N, E, Ty, Ix, Dn, De, L>,
        idx: NodeIndex<Ix>,
    );
}
