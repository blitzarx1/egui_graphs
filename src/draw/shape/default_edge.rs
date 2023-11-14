use petgraph::{stable_graph::IndexType, EdgeType};

use crate::{elements::EdgeID, Edge, Graph};

use super::Interactable;

#[derive(Clone, Debug)]
pub struct DefaultEdgeShape<Ix: IndexType> {
    pub edge_id: EdgeID<Ix>,

    pub width: f32,
    pub tip_size: f32,
    pub tip_angle: f32,
    pub curve_size: f32,
}

impl<E: Clone, Ix: IndexType> From<Edge<E, Ix>> for DefaultEdgeShape<Ix> {
    fn from(value: Edge<E, Ix>) -> Self {
        Self {
            edge_id: value.id(),
            width: value.width(),
            tip_size: value.tip_size(),
            tip_angle: value.tip_angle(),
            curve_size: value.curve_size(),
        }
    }
}

// impl<Ix: IndexType> Interactable for DefaultEdgeShape<Ix> {
//     fn is_inside<N: Clone, E: Clone, Ty: EdgeType, Ixx: IndexType>(
//         &self,
//         g: &Graph<N, E, Ty, Ixx>,
//         pos: egui::Pos2,
//     ) -> bool {
//         let (idx_start, idx_end) = g.edge_endpoints(self.edge_id.idx).unwrap();
//         let start = g.node_by_id(edge.start()).unwrap();
//         let end = g.node_by_id(edge.end()).unwrap();

//         is_inside_edge(start.location(), end.location(), self.width, pos)
//     }
// }
