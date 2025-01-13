use std::collections::HashSet;

use egui::Pos2;
use petgraph::stable_graph::{IndexType, NodeIndex};
use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::{
    layouts::{Layout, LayoutEvent, LayoutState},
    Graph,
};

const SPAWN_SIZE: f32 = 250.;

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct State {
    triggered_once: bool,
}

impl LayoutState for State {}

/// Randomly places nodes on the canvas. Does not override existing locations. Applies once.
#[derive(Debug, Default)]
pub struct Random {
    state: State,
}

impl Layout<State> for Random {
    fn next<N, E, Ty, Ix, Dn, De>(
        &mut self,
        g: &mut Graph<N, E, Ty, Ix, Dn, De>,
        not_placed: &HashSet<NodeIndex<Ix>>,
    ) where
        N: Clone,
        E: Clone,
        Ty: petgraph::EdgeType,
        Ix: IndexType,
        Dn: crate::DisplayNode<N, E, Ty, Ix>,
        De: crate::DisplayEdge<N, E, Ty, Ix, Dn>,
    {
        if self.state.triggered_once {
            return;
        }

        let mut rng = rand::thread_rng();
        for node in g.nodes_mut() {
            let idx = NodeIndex::new(node.id().index());
            if !not_placed.contains(&idx) {
                continue;
            };

            node.set_location(Pos2::new(
                rng.gen_range(0. ..SPAWN_SIZE),
                rng.gen_range(0. ..SPAWN_SIZE),
            ));
        }
    }

    fn state(&self) -> State {
        self.state.clone()
    }

    fn from_state(mut state: State, last_events: &[LayoutEvent]) -> impl Layout<State> {
        for event in last_events {
            if *event == LayoutEvent::NextCalledOnce {
                state.triggered_once = true;
            }
        }

        Self { state }
    }
}
