use egui::{Pos2, Shape};
use petgraph::{stable_graph::IndexType, EdgeType};

use crate::{draw::custom::DrawContext, Edge, Node};

pub trait NodeGraphDisplay<N: Clone, E: Clone, Ty: EdgeType, Ix: IndexType>: Interactable {
    /// Returns the closest point on the shape boundary to the provided `pos`.
    ///
    /// * `pos` - position is in the canvas coordinates.
    ///
    /// Could be used to snap the edge ends to the node.
    fn closest_boundary_point(&self, ctx: &DrawContext<N, E, Ty, Ix>, pos: Pos2) -> Pos2;
    fn from_node(node: &Node<N>) -> Self;
    fn shape(&self, ctx: &DrawContext<N, E, Ty, Ix>) -> Vec<Shape>;
}
pub trait EdgeGraphDisplay<N: Clone, E: Clone, Ty: EdgeType, Ix: IndexType>: Interactable {
    fn from_edge(edge: &Edge<E>) -> Self;
    fn shape(&self, ctx: &DrawContext<N, E, Ty, Ix>) -> Vec<Shape>;
}

pub trait Interactable {
    /// Checks if the provided `pos` is inside the shape.
    ///
    /// * `pos` - position is in the canvas coordinates.
    ///
    /// Could be used to bind mouse events to the custom drawn nodes.
    fn is_inside(&self, pos: Pos2) -> bool;
}
