use egui::util::id_type_map::SerializableAny;
use petgraph::{stable_graph::IndexType, EdgeType};

use crate::{DisplayEdge, DisplayNode, Graph};

pub trait LayoutState: SerializableAny + Default {}

pub trait Layout<S>: Default
where
    S: LayoutState,
{
    /// Creates a new layout from the given state. State is loaded and saved on every frame.
    fn from_state(state: S) -> impl Layout<S>;

    /// Called on every frame. It should update the graph layout aka nodes locations.
    fn next<N, E, Ty, Ix, Dn, De>(&mut self, g: &mut Graph<N, E, Ty, Ix, Dn, De>)
    where
        N: Clone,
        E: Clone,
        Ty: EdgeType,
        Ix: IndexType,
        Dn: DisplayNode<N, E, Ty, Ix>,
        De: DisplayEdge<N, E, Ty, Ix, Dn>;

    /// Returns the current state of the layout.
    fn state(&self) -> S;
}
