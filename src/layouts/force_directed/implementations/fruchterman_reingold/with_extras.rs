use crate::{
    layouts::force_directed::extras::ExtrasTuple, CenterGravity, DisplayEdge, DisplayNode, Extra,
    ForceAlgorithm, Graph,
};
use egui::{Rect, Vec2};
use petgraph::EdgeType;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use super::core::{
    apply_displacements, compute_attraction, compute_repulsion, prepare_constants,
    FruchtermanReingoldState,
};
use crate::layouts::layout::AnimatedState;
use crate::layouts::LayoutState;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(bound(serialize = "E: Serialize", deserialize = "E: DeserializeOwned"))]
pub struct FruchtermanReingoldWithExtrasState<E: ExtrasTuple> {
    pub base: FruchtermanReingoldState,
    pub extras: E,
}
impl<E: ExtrasTuple> LayoutState for FruchtermanReingoldWithExtrasState<E> {}

impl<E: ExtrasTuple> AnimatedState for FruchtermanReingoldWithExtrasState<E> {
    fn is_running(&self) -> bool {
        self.base.is_running
    }
    fn set_running(&mut self, v: bool) {
        self.base.is_running = v;
    }
    fn last_avg_displacement(&self) -> Option<f32> {
        self.base.last_avg_displacement
    }
    fn set_last_avg_displacement(&mut self, v: Option<f32>) {
        self.base.last_avg_displacement = v;
    }
}

#[derive(Debug, Default)]
pub struct FruchtermanReingoldWithExtras<E: ExtrasTuple> {
    state: FruchtermanReingoldWithExtrasState<E>,
    // Reusable displacement buffer
    scratch_disp: Vec<Vec2>,
}

impl<E: ExtrasTuple> FruchtermanReingoldWithExtras<E> {
    pub fn from_state(state: FruchtermanReingoldWithExtrasState<E>) -> Self {
        Self {
            state,
            scratch_disp: Vec::new(),
        }
    }
}

impl<E: ExtrasTuple> ForceAlgorithm for FruchtermanReingoldWithExtras<E> {
    type State = FruchtermanReingoldWithExtrasState<E>;

    fn from_state(state: Self::State) -> Self {
        Self {
            state,
            scratch_disp: Vec::new(),
        }
    }

    fn step<N, Ed, Ty, Ix, Dn, De>(&mut self, g: &mut Graph<N, Ed, Ty, Ix, Dn, De>, view: Rect)
    where
        N: Clone,
        Ed: Clone,
        Ty: EdgeType,
        Ix: petgraph::csr::IndexType,
        Dn: DisplayNode<N, Ed, Ty, Ix>,
        De: DisplayEdge<N, Ed, Ty, Ix, Dn>,
    {
        if g.node_count() == 0 || !self.state.base.is_running {
            return;
        }
        let base = &self.state.base;
        let area_rect = view;
        let Some(k) = prepare_constants(view, g.node_count(), base.k_scale) else {
            return;
        };

        let indices: Vec<_> = g.g().node_indices().collect();
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
            base.epsilon,
            base.c_repulse,
        );
        compute_attraction(
            g,
            &indices,
            &mut self.scratch_disp,
            k,
            base.epsilon,
            base.c_attract,
        );

        self.state
            .extras
            .apply_all(g, &indices, &mut self.scratch_disp, area_rect, k);

        let avg = apply_displacements(
            g,
            &indices,
            &self.scratch_disp,
            base.dt,
            base.damping,
            base.max_step,
        );
        self.state.base.last_avg_displacement = avg;
    }

    fn state(&self) -> Self::State {
        self.state.clone()
    }
}

/// Convenience aliases when only center gravity is desired.
pub type FruchtermanReingoldWithCenterGravity =
    FruchtermanReingoldWithExtras<(Extra<CenterGravity, true>, ())>;
pub type FruchtermanReingoldWithCenterGravityState =
    FruchtermanReingoldWithExtrasState<(Extra<CenterGravity, true>, ())>;
