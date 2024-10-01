use egui::Pos2;
use petgraph::stable_graph::IndexType;
use rand::Rng;

use crate::Graph;

use super::Layout;

const SPAWN_SIZE: f32 = 250.;

/// Randomly places nodes on the canvas. Does not override existing locations. Applies once.
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

        let mut rng = rand::thread_rng();
        for node in g.g.node_weights_mut() {
            node.set_location(Pos2::new(
                rng.gen_range(0. ..SPAWN_SIZE),
                rng.gen_range(0. ..SPAWN_SIZE),
            ));
        }

        self.triggered = true;
    }
}
