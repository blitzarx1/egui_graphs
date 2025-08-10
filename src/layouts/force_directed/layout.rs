use petgraph::{csr::IndexType, EdgeType};
use serde::{Deserialize, Serialize};

use crate::{
    layouts::{Layout, LayoutState},
    DisplayEdge, DisplayNode, Graph,
};
use egui::{Rect, Vec2};
use petgraph::stable_graph::NodeIndex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct State {
    /// Whether the simulation steps are applied. When false, positions remain static.
    pub is_running: bool,
    /// Euler integration timestep scalar (higher = faster but less stable).
    pub dt: f32,
    /// Minimum distance clamp to avoid division by zero in force calculations.
    pub epsilon: f32,
    /// Base gravity strength pulling nodes toward the view center (scaled by view size).
    pub gravity_base: f32,
    /// Damping factor applied to velocity each step (1 = none, 0 = full stop).
    pub damping: f32,
    /// Maximum pixel displacement allowed per frame (prevents large jumps / explosions).
    pub max_step: f32,
}

impl LayoutState for State {}

impl Default for State {
    fn default() -> Self {
        State {
            is_running: true,
            dt: 0.05,
            epsilon: 1e-3,
            gravity_base: 500.0,
            damping: 0.3,
            max_step: 10.0,
        }
    }
}

impl State {
    #[allow(dead_code)]
    pub fn with_params(
        is_running: bool,
        dt: f32,
        epsilon: f32,
        gravity_base: f32,
        damping: f32,
        max_step: f32,
    ) -> Self {
        Self {
            is_running,
            dt,
            epsilon,
            gravity_base,
            damping,
            max_step,
        }
    }
}

/// Force-directed layout (naive baseline).
///
/// This implementation trades performance for clarity:
/// * O(n²) all-pairs repulsion (no spatial partitioning / Barnes–Hut yet)
/// * Simple Euler integration (no adaptive timestep or higher-order solver)
/// * Uniform damping factor instead of a cooling schedule
/// * Clamps per-frame displacement (`max_step`) to reduce instability “explosions”
///
/// The tunable parameters are exposed through [`State`]. In real applications you usually
/// construct / mutate a `State` value yourself (e.g. serialize, load, tweak, then call
/// [`GraphView::set_layout_state`]) rather than relying on the demo UI. The demo example
/// only illustrates one possible control surface. This implementation is a reference
/// and will be improved (performance, stability, configurability) before a future
/// stable release.
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

        let params = &self.state;
        let Some((k, centre, g_strength)) =
            prepare_constants(canvas, g.node_count(), params.gravity_base)
        else {
            return;
        };

        // indices & displacement buffer
        let indices: Vec<_> = g.g().node_indices().collect();
        let mut disp: Vec<Vec2> = vec![Vec2::ZERO; indices.len()];

        compute_repulsion(g, &indices, &mut disp, k, params.epsilon);
        compute_attraction_and_gravity(
            g,
            &indices,
            &mut disp,
            k,
            centre,
            g_strength,
            params.epsilon,
        );
        apply_displacements(
            g,
            &indices,
            &disp,
            params.dt,
            params.damping,
            params.max_step,
        );
    }

    fn state(&self) -> State {
        self.state.clone()
    }
}

// --- Extracted helpers (pub(crate) for testing) ---------------------------------

pub(crate) fn prepare_constants(
    canvas: Rect,
    node_count: usize,
    gravity_base: f32,
) -> Option<(f32, egui::Pos2, f32)> {
    if node_count == 0 {
        return None;
    }
    let n = node_count as f32;
    let area = canvas.area().max(1.0);
    let k = (area / n).sqrt(); // ideal edge length
    let centre = canvas.center();
    let view_span = canvas.width().max(canvas.height()).max(1.0);
    let g_strength = gravity_base / view_span;
    if !k.is_finite() || !g_strength.is_finite() {
        return None;
    }
    Some((k, centre, g_strength))
}

pub(crate) fn compute_repulsion<N, E, Ty, Ix, Dn, De>(
    g: &Graph<N, E, Ty, Ix, Dn, De>,
    indices: &[NodeIndex<Ix>],
    disp: &mut [Vec2],
    k: f32,
    epsilon: f32,
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
            let dist = delta.length().max(epsilon);
            let force = (k * k) / dist;
            let dir = delta / dist; // unit vector
            disp[i] += dir * force;
            disp[j] -= dir * force; // equal & opposite
        }
    }
}

pub(crate) fn compute_attraction_and_gravity<N, E, Ty, Ix, Dn, De>(
    g: &Graph<N, E, Ty, Ix, Dn, De>,
    indices: &[NodeIndex<Ix>],
    disp: &mut [Vec2],
    k: f32,
    centre: egui::Pos2,
    g_strength: f32,
    epsilon: f32,
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
        // pull on each neighbour
        for nbr in g.g().neighbors_undirected(idx) {
            let delta = g.g().node_weight(nbr).unwrap().location() - loc;
            let dist = delta.length().max(epsilon);
            let force = (dist * dist) / k;
            disp[vec_pos] += (delta / dist) * force;
        }
        // mild gravity (inverse zoom-scaled)
        disp[vec_pos] += (centre - loc) * g_strength;
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

// --- Tests ---------------------------------------------------------------------
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
        // assign initial locations (spread)
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
        // place very close
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
        let params = State::default();
        let (k, centre, g_strength) = prepare_constants(rect, 2, params.gravity_base).unwrap();
        let indices: Vec<_> = g.g().node_indices().collect();
        let mut disp = vec![Vec2::ZERO; indices.len()];
        compute_repulsion(&g, &indices, &mut disp, k, params.epsilon);
        // disable gravity & attraction by not calling it
        // (we still could have gravity; ensure we don't apply it)
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
        // silence unused vars
        let _ = (centre, g_strength);
    }

    #[test]
    fn attraction_decreases_distance_when_far() {
        let mut g = make_graph(2);
        let idxs: Vec<_> = g.g().node_indices().collect();
        // connect nodes using Graph API
        g.add_edge(idxs[0], idxs[1], ());
        // place far apart
        g.g_mut()
            .node_weight_mut(idxs[0])
            .unwrap()
            .set_location(Pos2::new(0.0, 0.0));
        g.g_mut()
            .node_weight_mut(idxs[1])
            .unwrap()
            .set_location(Pos2::new(1200.0, 0.0));
        let rect = empty_ui_rect();
        let params = State::default();
        let (k, centre, g_strength) = prepare_constants(rect, 2, params.gravity_base).unwrap();
        let indices: Vec<_> = g.g().node_indices().collect();
        let mut disp = vec![Vec2::ZERO; indices.len()];
        let start_dist = 1200.0;
        compute_repulsion(&g, &indices, &mut disp, k, params.epsilon); // repulsion small vs attraction at large dist
        compute_attraction_and_gravity(
            &g,
            &indices,
            &mut disp,
            k,
            centre,
            g_strength,
            params.epsilon,
        );
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
            "Distance should shrink due to attraction ({} < {})",
            new_dist,
            start_dist
        );
    }

    #[test]
    fn gravity_pulls_towards_centre() {
        let mut g = make_graph(1);
        let idxs: Vec<_> = g.g().node_indices().collect();
        g.g_mut()
            .node_weight_mut(idxs[0])
            .unwrap()
            .set_location(Pos2::new(0.0, 0.0));
        let rect = empty_ui_rect();
        let params = State::default();
        let (k, centre, g_strength) = prepare_constants(rect, 1, params.gravity_base).unwrap();
        let indices: Vec<_> = g.g().node_indices().collect();
        let mut disp = vec![Vec2::ZERO; indices.len()];
        compute_attraction_and_gravity(
            &g,
            &indices,
            &mut disp,
            k,
            centre,
            g_strength,
            params.epsilon,
        ); // only gravity applies
        apply_displacements(
            &mut g,
            &indices,
            &disp,
            params.dt,
            params.damping,
            params.max_step,
        );
        let loc = g.g().node_weight(indices[0]).unwrap().location();
        let start_dist = (centre - Pos2::new(0.0, 0.0)).length();
        let new_dist = (centre - loc).length();
        assert!(
            new_dist < start_dist,
            "Node should move closer to centre ({} < {})",
            new_dist,
            start_dist
        );
    }
}
