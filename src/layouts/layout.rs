use petgraph::{stable_graph::IndexType, EdgeType};

use crate::{DisplayEdge, DisplayNode, Graph};

pub trait Layout: Default {
    /// This function is called on every frame. It should update the graph layout aka nodes locations.
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
}
