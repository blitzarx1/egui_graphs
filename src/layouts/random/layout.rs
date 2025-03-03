use egui::Pos2;
use petgraph::stable_graph::IndexType;
use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::{
    layouts::{Layout, LayoutState},
    Graph,
};
const SPAWN_SIZE: f32 = 250.;

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct State {
    triggered: bool,
}

impl LayoutState for State {}

/// Randomly places nodes on the canvas. Does not override existing locations. Applies once.
#[derive(Debug, Default)]
pub struct Random {
    state: State,
}

impl Layout<State> for Random {
    fn next<N, E, Ty, Ix, Dn, De>(&mut self, g: &mut Graph<N, E, Ty, Ix, Dn, De>)
    where
        N: Clone,
        E: Clone,
        Ty: petgraph::EdgeType,
        Ix: IndexType,
        Dn: crate::DisplayNode<N, E, Ty, Ix>,
        De: crate::DisplayEdge<N, E, Ty, Ix, Dn>,
    {
        if self.state.triggered {
            return;
        }

        let mut rng = rand::rng();
        for node in g.g.node_weights_mut() {
            node.set_layout_location(Pos2::new(
                rng.random_range(0. ..SPAWN_SIZE),
                rng.random_range(0. ..SPAWN_SIZE),
            ));
        }

        self.state.triggered = true;
    }

    fn state(&self) -> State {
        self.state.clone()
    }

    fn from_state(state: State) -> impl Layout<State> {
        Self { state }
    }
}
