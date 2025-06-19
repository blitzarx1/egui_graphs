use egui::Vec2;
use petgraph::{csr::IndexType, EdgeType};
use serde::{Deserialize, Serialize};

use crate::{
    layouts::{Layout, LayoutState},
    DisplayEdge, DisplayNode, Graph,
};

const DT: f32 = 0.05;
const GRAVITY: f32 = 3.0;
const EPSILON: f32 = 0.001;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct State {
    is_running: bool,
}

impl LayoutState for State {}

impl Default for State {
    fn default() -> Self {
        State { is_running: true }
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

    fn next<N, E, Ty, Ix, Dn, De>(&mut self, g: &mut Graph<N, E, Ty, Ix, Dn, De>)
    where
        N: Clone,
        E: Clone,
        Ty: EdgeType,
        Ix: IndexType,
        Dn: DisplayNode<N, E, Ty, Ix>,
        De: DisplayEdge<N, E, Ty, Ix, Dn>,
    {
        if !self.state.is_running || g.node_count() == 0 {
            return;
        }

        /* ----------------------------------------------------------------- */
        /*                         pre-computed values                       */
        /* ----------------------------------------------------------------- */
        let n = g.node_count() as f32;
        let area = g.bounds().area().max(1.0);
        let k = (area / n).sqrt(); // ideal edge length
        let centre = g.bounds().center();

        let indices: Vec<_> = g.g().node_indices().collect();
        let mut disp: Vec<Vec2> = vec![Vec2::ZERO; indices.len()];

        /* ----------------------------------------------------------------- */
        /*           PASS 1 — node-to-node repulsion (O(|V|²))               */
        /* ----------------------------------------------------------------- */
        for i in 0..indices.len() {
            for j in (i + 1)..indices.len() {
                let idx_i = indices[i];
                let idx_j = indices[j];

                let delta = g.g().node_weight(idx_i).unwrap().location()
                    - g.g().node_weight(idx_j).unwrap().location();
                let dist = delta.length().max(EPSILON); // no division by 0

                let force = (k * k) / dist;
                let dir = delta / dist; // unit vector

                disp[i] += dir * force; // push i
                disp[j] -= dir * force; // equal & opposite push j
            }
        }

        /* ----------------------------------------------------------------- */
        /*           PASS 2 — edge attraction  +  centre gravity             */
        /* ----------------------------------------------------------------- */
        for (idx_pos, &idx) in indices.iter().enumerate() {
            let loc = g.g().node_weight(idx).unwrap().location();

            // attract towards every neighbour
            for nbr in g.g().neighbors_undirected(idx) {
                let delta = g.g().node_weight(nbr).unwrap().location() - loc;
                let dist = delta.length().max(EPSILON);

                let force = (dist * dist) / k;
                disp[idx_pos] += (delta / dist) * force;
            }

            // gentle gravity to centre
            disp[idx_pos] += (centre - loc) * GRAVITY;
        }

        /* ----------------------------------------------------------------- */
        /*                    integrate & write back                         */
        /* ----------------------------------------------------------------- */
        for (idx_pos, &idx) in indices.iter().enumerate() {
            let loc = g.g().node_weight(idx).unwrap().location();
            let new_loc = loc + disp[idx_pos] * DT;

            g.g_mut()
                .node_weight_mut(idx)
                .unwrap()
                .set_location(new_loc);
        }
    }
    fn state(&self) -> State {
        self.state.clone()
    }
}
