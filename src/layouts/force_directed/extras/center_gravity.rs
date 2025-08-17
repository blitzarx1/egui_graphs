use egui::{Rect, Vec2};
use serde::{Deserialize, Serialize};

use super::core::ExtraForce;
use crate::{DisplayEdge, DisplayNode, Graph};
use petgraph::EdgeType;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CenterGravityParams {
    pub c: f32,
}
impl Default for CenterGravityParams {
    fn default() -> Self {
        Self { c: 0.3 }
    }
}

#[derive(Debug, Default)]
pub struct CenterGravity;

impl ExtraForce for CenterGravity {
    type Params = CenterGravityParams;

    fn apply<N, E, Ty, Ix, Dn, De>(
        params: &Self::Params,
        g: &Graph<N, E, Ty, Ix, Dn, De>,
        indices: &[petgraph::stable_graph::NodeIndex<Ix>],
        disp: &mut [Vec2],
        area: Rect,
        _k: f32,
    ) where
        N: Clone,
        E: Clone,
        Ty: EdgeType,
        Ix: petgraph::csr::IndexType,
        Dn: DisplayNode<N, E, Ty, Ix>,
        De: DisplayEdge<N, E, Ty, Ix, Dn>,
    {
        if params.c == 0.0 {
            return;
        }
        let center = area.center();
        for (vec_pos, &idx) in indices.iter().enumerate() {
            let pos = g.g().node_weight(idx).unwrap().location();
            let delta = center - pos;
            disp[vec_pos] += delta * params.c;
        }
    }
}
