use egui::{Pos2, Shape};
use petgraph::{stable_graph::IndexType, EdgeType};

use crate::{draw::drawer::DrawContext, Edge, Graph, Node};

pub trait NodeDisplay<N: Clone, E: Clone, Ty: EdgeType, Ix: IndexType>:
    Interactable<N, E, Ty, Ix> + From<Node<N, Ix>>
{
    /// Returns the closest point on the shape boundary to the provided `pos`.
    ///
    /// * `pos` - position is in the canvas coordinates.
    ///
    /// Could be used to snap the edge ends to the node.
    fn closest_boundary_point(&self, pos: Pos2) -> Pos2;

    /// Draws shapes of the node.
    ///
    /// * `ctx` - should be used to determine current global properties.
    ///
    /// Use `ctx.meta` to properly scale and translate the shape.
    fn shapes(&self, ctx: &DrawContext<N, E, Ty, Ix>) -> Vec<Shape>;
}
pub trait EdgeDisplay<N: Clone, E: Clone, Ty: EdgeType, Ix: IndexType>:
    Interactable<N, E, Ty, Ix> + From<Edge<E, Ix>>
{
    /// Draws shapes of the edge.
    ///
    /// * `ctx` - should be used to determine current global properties.
    /// * `start` and `end` - start and end points of the edge.
    ///
    /// Use `ctx.meta` to properly scale and translate the shape.
    ///
    /// Get [NodeGraphDisplay] from node endpoints to get start and end coordinates using [closest_boundary_point](NodeGraphDisplay::closest_boundary_point).
    fn shapes(
        &self,
        start: Node<N, Ix>,
        end: Node<N, Ix>,
        ctx: &DrawContext<N, E, Ty, Ix>,
    ) -> Vec<Shape>;
}

pub trait Interactable<N: Clone, E: Clone, Ty: EdgeType, Ix: IndexType> {
    /// Checks if the provided `pos` is inside the shape.
    ///
    /// * `pos` - position is in the canvas coordinates.
    ///
    /// Could be used to bind mouse events to the custom drawn nodes.
    fn is_inside(&self, g: &Graph<N, E, Ty, Ix>, pos: Pos2) -> bool;
}
