use crate::Graph;

use super::Layout;

#[derive(Default)]
pub struct Hierarchical {}

impl Layout for Hierarchical {
    fn next<
        N: Clone,
        E: Clone,
        Ty: petgraph::EdgeType,
        Ix: petgraph::csr::IndexType,
        Dn: crate::DisplayNode<N, E, Ty, Ix>,
        De: crate::DisplayEdge<N, E, Ty, Ix, Dn>,
        L: Layout,
    >(
        &mut self,
        g: &mut Graph<N, E, Ty, Ix, Dn, De, L>,
    ) {
        todo!()
    }
}
