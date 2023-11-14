use petgraph::stable_graph::IndexType;

use crate::{elements::EdgeID, Edge};

#[derive(Clone, Debug)]
pub struct DefaultEdgeShape<Ix: IndexType> {
    pub edge_id: EdgeID<Ix>,

    pub width: f32,
    pub tip_size: f32,
    pub tip_angle: f32,
    pub curve_size: f32,
}