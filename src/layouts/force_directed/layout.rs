use egui::Vec2;
use serde::{Deserialize, Serialize};

use crate::layouts::{Layout, LayoutState};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct State {
    is_running: bool,
    dt: f32,
}

impl LayoutState for State {}

impl Default for State {
    fn default() -> Self {
        State {
            is_running: true,
            dt: 0.016, // Default to 60 FPS
        }
    }
}

#[derive(Debug, Default)]
pub struct ForceDirected {
    state: State,
}

impl Layout<State> for ForceDirected {
    fn from_state(state: State) -> impl Layout<State> {
        ForceDirected { state }
    }

    fn next<N, E, Ty, Ix, Dn, De>(&mut self, g: &mut crate::Graph<N, E, Ty, Ix, Dn, De>)
    where
        N: Clone,
        E: Clone,
        Ty: petgraph::EdgeType,
        Ix: petgraph::csr::IndexType,
        Dn: crate::DisplayNode<N, E, Ty, Ix>,
        De: crate::DisplayEdge<N, E, Ty, Ix, Dn>,
    {
        if !self.state.is_running {
            return;
        }

        let speed = Vec2::new(1.0, 1.0);

        g.g.node_weights_mut().for_each(|n| {
            let dx = speed * self.state.dt;

            let new_loc = n.location() + dx;
            n.set_location(new_loc);
        });
    }

    fn state(&self) -> State {
        self.state.clone()
    }
}
