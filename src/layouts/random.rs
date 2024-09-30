use egui::Pos2;
use petgraph::stable_graph::{IndexType, NodeIndex};
use rand::Rng;

use crate::Graph;

use super::Layout;

const DEFAULT_SPAWN_SIZE: f32 = 250.;

#[derive(Default)]
pub struct Random {
    triggered: bool,
}

impl Layout for Random {
    fn next<
        N: Clone,
        E: Clone,
        Ty: petgraph::EdgeType,
        Ix: IndexType,
        Dn: crate::DisplayNode<N, E, Ty, Ix>,
        De: crate::DisplayEdge<N, E, Ty, Ix, Dn>,
        L: Layout,
    >(
        &mut self,
        g: &mut Graph<N, E, Ty, Ix, Dn, De, L>,
    ) {
        if self.triggered {
            return;
        }

        for node in g.g.node_weights_mut() {
            node.set_location(random_location(DEFAULT_SPAWN_SIZE));
        }

        self.triggered = true;
    }

    fn next_for_node<
        N: Clone,
        E: Clone,
        Ty: petgraph::EdgeType,
        Ix: IndexType,
        Dn: crate::DisplayNode<N, E, Ty, Ix>,
        De: crate::DisplayEdge<N, E, Ty, Ix, Dn>,
        L: Layout,
    >(
        &mut self,
        g: &mut Graph<N, E, Ty, Ix, Dn, De, L>,
        idx: NodeIndex<Ix>,
    ) {
        todo!()
    }
}

fn random_location(size: f32) -> Pos2 {
    let mut rng = rand::thread_rng();
    Pos2::new(rng.gen_range(0. ..size), rng.gen_range(0. ..size))
}
