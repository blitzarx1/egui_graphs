use petgraph::{csr::IndexType, EdgeType};
use serde::{Deserialize, Serialize};

use crate::{
    layouts::{Layout, LayoutState},
    DisplayEdge, DisplayNode, Graph,
};

const DT: f32 = 0.05; // Euler step
const EPSILON: f32 = 1e-3; // avoid /0
const GRAVITY_BASE: f32 = 500.0; // bigger → stronger “centre” pull
const DAMPING: f32 = 0.9; // 1.0 = no damping, 0.0 = freeze instantly
const MAX_STEP: f32 = 10.0; // clamp displacement per frame (pixels)

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

    fn next<N, E, Ty, Ix, Dn, De>(&mut self, g: &mut Graph<N, E, Ty, Ix, Dn, De>, ui: &egui::Ui)
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

        let canvas = ui.ctx().screen_rect();

        let n = g.node_count() as f32;
        let area = canvas.area().max(1.0);
        let k = (area / n).sqrt(); // ideal edge length
        let centre = canvas.center();
        let view_span = canvas.width().max(canvas.height()).max(1.0);
        let g_strength = GRAVITY_BASE / view_span;

        if !k.is_finite() || !g_strength.is_finite() {
            return;
        }

        /* -------- gather indices & init displacement accumulator ---------------- */
        let indices: Vec<_> = g.g().node_indices().collect();
        let mut disp: Vec<egui::Vec2> = vec![egui::Vec2::ZERO; indices.len()];

        /* ---------------- PASS 1 : universal repulsion  ------------------------- */
        for i in 0..indices.len() {
            for j in (i + 1)..indices.len() {
                let (idx_i, idx_j) = (indices[i], indices[j]);

                let delta = g.g().node_weight(idx_i).unwrap().location()
                    - g.g().node_weight(idx_j).unwrap().location();
                let dist = delta.length().max(EPSILON);

                let force = (k * k) / dist;
                let dir = delta / dist; // unit vector

                disp[i] += dir * force;
                disp[j] -= dir * force; // equal & opposite
            }
        }

        /* -------- PASS 2 : edge attraction + gravity towards centre -------------- */
        for (vec_pos, &idx) in indices.iter().enumerate() {
            let loc = g.g().node_weight(idx).unwrap().location();

            // pull on each neighbour
            for nbr in g.g().neighbors_undirected(idx) {
                let delta = g.g().node_weight(nbr).unwrap().location() - loc;
                let dist = delta.length().max(EPSILON);

                let force = (dist * dist) / k;
                disp[vec_pos] += (delta / dist) * force;
            }

            // mild gravity (inverse zoom-scaled)
            disp[vec_pos] += (centre - loc) * g_strength;
        }

        /* ------------ integrate, damp, clamp, and write positions ---------------- */
        for (vec_pos, &idx) in indices.iter().enumerate() {
            // velocity damping
            let mut step = disp[vec_pos] * DT * DAMPING;

            // clamp huge jumps
            if step.length() > MAX_STEP {
                step = step.normalized() * MAX_STEP;
            }

            // apply
            let loc = g.g().node_weight(idx).unwrap().location();
            let new_loc = loc + step;

            if !new_loc.x.is_finite() || !new_loc.y.is_finite() {
                continue;
            }

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
