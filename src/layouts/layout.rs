use std::collections::HashSet;

use egui::util::id_type_map::SerializableAny;
use petgraph::{
    stable_graph::{IndexType, NodeIndex},
    EdgeType,
};
use serde::{Deserialize, Serialize};

use crate::{DisplayEdge, DisplayNode, Graph};

/// `LayoutState` is persisted and loads on every frame.
/// Should be as lightweight as possible with fields representing
/// some flags needed for layout initialization.
pub trait LayoutState: SerializableAny + Default {}

pub trait Layout<S>: Default
where
    S: LayoutState,
{
    /// Initializes layout from state.
    fn from_state(state: S, last_events: &[LayoutEvent]) -> impl Layout<S>;

    /// Called on every frame. It should update the graph layout in other words
    /// updates nodes locations.
    fn next<N, E, Ty, Ix, Dn, De>(
        &mut self,
        g: &mut Graph<N, E, Ty, Ix, Dn, De>,
        not_placed: &HashSet<NodeIndex<Ix>>,
    ) where
        N: Clone,
        E: Clone,
        Ty: EdgeType,
        Ix: IndexType,
        Dn: DisplayNode<N, E, Ty, Ix>,
        De: DisplayEdge<N, E, Ty, Ix, Dn>;

    /// Returns the current state of the layout.
    fn state(&self) -> S;
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum LayoutEvent {
    /// Means that `next` func is called at least once. Used to indicate firs call of next func.
    NextCalledOnce,
    /// Means that `next` func is called
    NextCalled,
}
