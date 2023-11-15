use egui::{Pos2, Shape, Vec2};
use petgraph::{stable_graph::IndexType, EdgeType};

use crate::{draw::drawer::DrawContext, Edge, Graph, Node};

pub trait DisplayNode<N, E, Ty, Ix>: Interactable<N, E, Ty, Ix> + From<Node<N, Ix>>
where
    N: Clone,
    E: Clone,
    Ty: EdgeType,
    Ix: IndexType,
{
    /// Returns the closest point on the shape boundary in the direction of dir.
    ///
    /// * `dir` - direction pointing from the shape center to the required boundary point.
    ///
    /// Could be used to snap the edge ends to the node.
    fn closest_boundary_point(&self, dir: Vec2) -> Pos2;

    /// Draws shapes of the node.
    ///
    /// * `ctx` - should be used to determine current global properties.
    ///
    /// Use `ctx.meta` to properly scale and translate the shape.
    fn shapes(&self, ctx: &DrawContext<N, E, Ty, Ix>) -> Vec<Shape>;
}
pub trait DisplayEdge<N, E, Ty, Ix>: Interactable<N, E, Ty, Ix> + From<Edge<E, Ix>>
where
    N: Clone,
    E: Clone,
    Ty: EdgeType,
    Ix: IndexType,
{
    /// Draws shapes of the edge.
    ///
    /// * `ctx` - should be used to determine current global properties.
    /// * `start` and `end` - start and end points of the edge.
    ///
    /// Use `ctx.meta` to properly scale and translate the shape.
    ///
    /// Get [NodeGraphDisplay] from node endpoints to get start and end coordinates using [closest_boundary_point](NodeGraphDisplay::closest_boundary_point).
    fn shapes<Dn: DisplayNode<N, E, Ty, Ix>>(&self, ctx: &DrawContext<N, E, Ty, Ix>) -> Vec<Shape>;
}

pub trait Interactable<N: Clone, E: Clone, Ty: EdgeType, Ix: IndexType> {
    /// Checks if the provided `pos` is inside the shape.
    ///
    /// * `pos` - position is in the canvas coordinates.
    ///
    /// Could be used to bind mouse events to the custom drawn nodes.
    fn is_inside<Dn: DisplayNode<N, E, Ty, Ix>>(&self, g: &Graph<N, E, Ty, Ix>, pos: Pos2) -> bool;
}
