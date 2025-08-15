use crate::{DisplayEdge, DisplayNode, ForceAlgorithm, Graph};
use egui::{Rect, Vec2};
use petgraph::{csr::IndexType, stable_graph::NodeIndex, EdgeType};
use serde::{Deserialize, Serialize};

use crate::layouts::LayoutState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FruchtermanReingoldState {
    pub is_running: bool,
    pub dt: f32,
    pub epsilon: f32,
    pub damping: f32,
    pub max_step: f32,
    pub k_scale: f32,
    pub c_attract: f32,
    pub c_repulse: f32,
}

impl LayoutState for FruchtermanReingoldState {}

impl Default for FruchtermanReingoldState {
    fn default() -> Self {
        FruchtermanReingoldState {
            is_running: true,
            dt: 0.05,
            epsilon: 1e-3,
            damping: 0.3,
            max_step: 10.0,
            k_scale: 1.0,
            c_attract: 1.0,
            c_repulse: 1.0,
        }
    }
}

impl FruchtermanReingoldState {
    #[allow(dead_code)]
    #[allow(clippy::too_many_arguments)]
    pub fn with_params(
        is_running: bool,
        dt: f32,
        epsilon: f32,
        damping: f32,
        max_step: f32,
        k_scale: f32,
        c_attract: f32,
        c_repulse: f32,
    ) -> Self {
        Self {
            is_running,
            dt,
            epsilon,
            damping,
            max_step,
            k_scale,
            c_attract,
            c_repulse,
        }
    }
}

#[derive(Debug, Default)]
pub struct FruchtermanReingold {
    state: FruchtermanReingoldState,
    // Reusable displacement buffer to avoid per-frame allocations
    scratch_disp: Vec<Vec2>,
}

impl FruchtermanReingold {
    pub fn from_state(state: FruchtermanReingoldState) -> Self {
        Self {
            state,
            scratch_disp: Vec::new(),
        }
    }
}

impl ForceAlgorithm for FruchtermanReingold {
    type State = FruchtermanReingoldState;

    fn from_state(state: Self::State) -> Self {
        Self {
            state,
            scratch_disp: Vec::new(),
        }
    }

    fn step<N, E, Ty, Ix, Dn, De>(&mut self, g: &mut Graph<N, E, Ty, Ix, Dn, De>, view: Rect)
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

        let params = &self.state;
        // Always compute k from the viewport area for stability and simplicity.
        let Some(k) = prepare_constants(view, g.node_count(), params.k_scale) else {
            return;
        };

        let indices: Vec<_> = g.g().node_indices().collect();
        // Ensure scratch buffer is sized and zeroed
        if self.scratch_disp.len() == indices.len() {
            self.scratch_disp.fill(Vec2::ZERO);
        } else {
            self.scratch_disp.resize(indices.len(), Vec2::ZERO);
        }

        compute_repulsion(
            g,
            &indices,
            &mut self.scratch_disp,
            k,
            params.epsilon,
            params.c_repulse,
        );
        compute_attraction(
            g,
            &indices,
            &mut self.scratch_disp,
            k,
            params.epsilon,
            params.c_attract,
        );
        apply_displacements(
            g,
            &indices,
            &self.scratch_disp,
            params.dt,
            params.damping,
            params.max_step,
        );
    }

    fn state(&self) -> Self::State {
        self.state.clone()
    }
}

pub(crate) fn prepare_constants(canvas: Rect, node_count: usize, k_scale: f32) -> Option<f32> {
    if node_count == 0 {
        return None;
    }
    let n = node_count as f32;
    let area = canvas.area().max(1.0);
    let k_ideal = (area / n).sqrt(); // ideal edge length
    let k = k_ideal * k_scale;
    if !k.is_finite() {
        return None;
    }
    Some(k)
}

pub(crate) fn compute_repulsion<N, E, Ty, Ix, Dn, De>(
    g: &Graph<N, E, Ty, Ix, Dn, De>,
    indices: &[NodeIndex<Ix>],
    disp: &mut [Vec2],
    k: f32,
    epsilon: f32,
    c_repulse: f32,
) where
    N: Clone,
    E: Clone,
    Ty: EdgeType,
    Ix: IndexType,
    Dn: DisplayNode<N, E, Ty, Ix>,
    De: DisplayEdge<N, E, Ty, Ix, Dn>,
{
    for i in 0..indices.len() {
        for j in (i + 1)..indices.len() {
            let (idx_i, idx_j) = (indices[i], indices[j]);
            let delta = g.g().node_weight(idx_i).unwrap().location()
                - g.g().node_weight(idx_j).unwrap().location();
            let distance = delta.length().max(epsilon);
            let force = c_repulse * (k * k) / distance;
            let dir = delta / distance;
            disp[i] += dir * force;
            disp[j] -= dir * force;
        }
    }
}

pub(crate) fn compute_attraction<N, E, Ty, Ix, Dn, De>(
    g: &Graph<N, E, Ty, Ix, Dn, De>,
    indices: &[NodeIndex<Ix>],
    disp: &mut [Vec2],
    k: f32,
    epsilon: f32,
    c_attract: f32,
) where
    N: Clone,
    E: Clone,
    Ty: EdgeType,
    Ix: IndexType,
    Dn: DisplayNode<N, E, Ty, Ix>,
    De: DisplayEdge<N, E, Ty, Ix, Dn>,
{
    for (vec_pos, &idx) in indices.iter().enumerate() {
        let loc = g.g().node_weight(idx).unwrap().location();
        for nbr in g.g().neighbors_undirected(idx) {
            let delta = g.g().node_weight(nbr).unwrap().location() - loc;
            let distance = delta.length().max(epsilon);
            let force = c_attract * (distance * distance) / k;
            disp[vec_pos] += (delta / distance) * force;
        }
    }
}

pub(crate) fn apply_displacements<N, E, Ty, Ix, Dn, De>(
    g: &mut Graph<N, E, Ty, Ix, Dn, De>,
    indices: &[NodeIndex<Ix>],
    disp: &[Vec2],
    dt: f32,
    damping: f32,
    max_step: f32,
) where
    N: Clone,
    E: Clone,
    Ty: EdgeType,
    Ix: IndexType,
    Dn: DisplayNode<N, E, Ty, Ix>,
    De: DisplayEdge<N, E, Ty, Ix, Dn>,
{
    for (vec_pos, &idx) in indices.iter().enumerate() {
        let mut step = disp[vec_pos] * dt * damping;
        if step.length() > max_step {
            step = step.normalized() * max_step;
        }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{to_graph, DefaultEdgeShape, DefaultNodeShape};
    use egui::{Pos2, Rect};
    use petgraph::stable_graph::StableGraph;

    fn empty_ui_rect() -> Rect {
        Rect::from_min_max(Pos2::new(0.0, 0.0), Pos2::new(1000.0, 1000.0))
    }

    fn make_graph(
        num: usize,
    ) -> Graph<
        (),
        (),
        petgraph::Directed,
        petgraph::stable_graph::DefaultIx,
        DefaultNodeShape,
        DefaultEdgeShape,
    > {
        let mut g: StableGraph<(), ()> = StableGraph::default();
        for _ in 0..num {
            g.add_node(());
        }
        let mut graph = to_graph(&g);
        let node_indices: Vec<_> = graph.g().node_indices().collect();
        for (i, idx) in node_indices.iter().enumerate() {
            let mut_loc = Pos2::new(i as f32 * 10.0, 0.0);
            graph
                .g_mut()
                .node_weight_mut(*idx)
                .unwrap()
                .set_location(mut_loc);
        }
        graph
    }

    #[test]
    fn repulsion_increases_distance() {
        let mut g = make_graph(2);
        let idxs: Vec<_> = g.g().node_indices().collect();
        g.g_mut()
            .node_weight_mut(idxs[0])
            .unwrap()
            .set_location(Pos2::new(0.0, 0.0));
        g.g_mut()
            .node_weight_mut(idxs[1])
            .unwrap()
            .set_location(Pos2::new(1.0, 0.0));
        let rect = empty_ui_rect();
        let params = FruchtermanReingoldState::default();
        let k = prepare_constants(rect, 2, params.k_scale).unwrap();
        let indices: Vec<_> = g.g().node_indices().collect();
        let mut disp = vec![Vec2::ZERO; indices.len()];
        compute_repulsion(&g, &indices, &mut disp, k, params.epsilon, params.c_repulse);
        apply_displacements(
            &mut g,
            &indices,
            &disp,
            params.dt,
            params.damping,
            params.max_step,
        );
        let a = g.g().node_weight(indices[0]).unwrap().location();
        let b = g.g().node_weight(indices[1]).unwrap().location();
        assert!((b.x - a.x).abs() > 1.0, "Nodes should move apart");
    }

    #[test]
    fn attraction_decreases_distance_when_far() {
        let mut g = make_graph(2);
        let idxs: Vec<_> = g.g().node_indices().collect();
        g.add_edge(idxs[0], idxs[1], ());
        g.g_mut()
            .node_weight_mut(idxs[0])
            .unwrap()
            .set_location(Pos2::new(0.0, 0.0));
        g.g_mut()
            .node_weight_mut(idxs[1])
            .unwrap()
            .set_location(Pos2::new(1200.0, 0.0));
        let rect = empty_ui_rect();
        let params = FruchtermanReingoldState::default();
        let k = prepare_constants(rect, 2, params.k_scale).unwrap();
        let indices: Vec<_> = g.g().node_indices().collect();
        let mut disp = vec![Vec2::ZERO; indices.len()];
        let start_dist = 1200.0;
        compute_repulsion(&g, &indices, &mut disp, k, params.epsilon, params.c_repulse);
        compute_attraction(&g, &indices, &mut disp, k, params.epsilon, params.c_attract);
        apply_displacements(
            &mut g,
            &indices,
            &disp,
            params.dt,
            params.damping,
            params.max_step,
        );
        let a = g.g().node_weight(indices[0]).unwrap().location();
        let b = g.g().node_weight(indices[1]).unwrap().location();
        let new_dist = (b - a).length();
        assert!(
            new_dist < start_dist,
            "Distance should shrink due to attraction"
        );
    }
}
