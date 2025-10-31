use petgraph::{csr::IndexType, EdgeType};

use crate::{layouts::Layout, DisplayEdge, DisplayNode, Graph};

use super::algorithm::ForceAlgorithm;

#[derive(Debug, Default)]
pub struct ForceDirected<A: ForceAlgorithm> {
    alg: A,
}

impl<A: ForceAlgorithm> Layout<A::State> for ForceDirected<A> {
    fn from_state(state: A::State) -> impl Layout<A::State> {
        Self {
            alg: A::from_state(state),
        }
    }

    fn next<N, E, Ty, Ix, Dn, De>(&mut self, g: &mut Graph<N, E, Ty, Ix, Dn, De>, ui: &egui::Ui)
    where
        N: Clone,
        E: Clone,
        Ty: EdgeType,
        Ix: IndexType,
        Dn: DisplayNode<N, E, Ty, Ix>,
        De: DisplayEdge<N, E, Ty, Ix, Dn>,
    {
        if g.node_count() == 0 {
            return;
        }

        self.alg.step(g, ui.ctx().content_rect());
    }

    fn state(&self) -> A::State {
        self.alg.state()
    }
}
