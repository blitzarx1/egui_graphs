use egui::{Pos2, Shape};
use petgraph::{stable_graph::IndexType, EdgeType};

use crate::draw::custom::DrawContext;

pub trait NodeGraphDisplay: Connectable + Interactable {}
pub trait EdgeGraphDisplay: Interactable {}

pub trait Interactable {
    /// Checks if the provided `pos` is inside the shape.
    ///
    /// * `pos` - position is in the canvas coordinates.
    ///
    /// Could be used to bind mouse events to the custom drawn nodes.
    fn is_inside(&self, pos: Pos2) -> bool;
}

pub trait Drawable<N: Clone, E: Clone, Ty: EdgeType, Ix: IndexType> {
    fn shape(&self, ctx: &DrawContext<N, E, Ty, Ix>) -> Vec<Shape>;
}

pub trait Connectable {
    /// Returns the closest point on the shape boundary to the provided `pos`.
    ///
    /// * `pos` - position is in the canvas coordinates.
    ///
    /// Could be used to snap the edge ends to the node.
    fn closest_boundary_point(&self, pos: Pos2) -> Pos2;
}
