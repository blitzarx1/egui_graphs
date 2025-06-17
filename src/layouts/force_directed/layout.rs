use egui::{Pos2, Vec2};
use serde::{Deserialize, Serialize};

use crate::layouts::{Layout, LayoutState};

const FORCE_CENTER_REPEL: f32 = 25.0;
const FORCE_NEIGHBOR_ATTR: f32 = 0.01;
const DT: f32 = 0.5;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct State {
    is_running: bool,
}

impl LayoutState for State {}

impl Default for State {
    fn default() -> Self {
        State {
            is_running: true,
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

        let center = find_center(g);
        let indices: Vec<_> = g.g.node_indices().collect();
        for idx in indices {
            let loc = g.g.node_weight(idx).unwrap().location();
            let vc = center - loc;
            let vc_len_sq = vc.length().powi(2);
            let dx_center = if vc_len_sq > 0.0 {
                -vc * (FORCE_CENTER_REPEL / vc_len_sq) * DT
            } else {
                Vec2::ZERO
            };

            let dx_neighbor =
                g.g.neighbors_undirected(idx)
                    .map(|nbr_idx| {
                        let vn = g.g.node_weight(nbr_idx).unwrap().location() - loc;
                        if vn.length() > 0.0 {
                            vn * FORCE_NEIGHBOR_ATTR * DT
                        } else {
                            Vec2::ZERO
                        }
                    })
                    .fold(Vec2::ZERO, |acc, v| acc + v);

            let new_loc = loc + dx_center + dx_neighbor;

            g.g.node_weight_mut(idx).unwrap().set_location(new_loc);
        }
    }

    fn state(&self) -> State {
        self.state.clone()
    }
}

fn find_center<N, E, Ty, Ix, Dn, De>(g: &crate::Graph<N, E, Ty, Ix, Dn, De>) -> Pos2
where
    N: Clone,
    E: Clone,
    Ty: petgraph::EdgeType,
    Ix: petgraph::csr::IndexType,
    Dn: crate::DisplayNode<N, E, Ty, Ix>,
    De: crate::DisplayEdge<N, E, Ty, Ix, Dn>,
{
    let mut min_x = f32::MAX;
    let mut max_x = f32::MIN;
    let mut min_y = f32::MAX;
    let mut max_y = f32::MIN;

    for node in g.g.node_weights() {
        let loc = node.location();
        if loc.x < min_x {
            min_x = loc.x;
        }
        if loc.x > max_x {
            max_x = loc.x;
        }
        if loc.y < min_y {
            min_y = loc.y;
        }
        if loc.y > max_y {
            max_y = loc.y;
        }
    }

    Pos2::new((min_x + max_x) / 2.0, (min_y + max_y) / 2.0)
}
