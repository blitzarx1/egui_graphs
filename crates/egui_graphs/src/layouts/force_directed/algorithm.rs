use crate::{DisplayEdge, DisplayNode, Graph};
use egui::Rect;
use petgraph::{csr::IndexType, EdgeType};

use super::super::layout::LayoutState;

/// A pluggable force-directed algorithm interface decoupled from the UI boilerplate.
///
/// The algorithm operates on a Graph and a viewport Rect and advances the layout by one step.
pub trait ForceAlgorithm: Default {
    type State: LayoutState + Clone;

    /// Construct from a state value (typically deserialized each frame).
    fn from_state(state: Self::State) -> Self;

    /// Advance the simulation by one step using the given viewport rectangle if needed.
    fn step<N, E, Ty, Ix, Dn, De>(&mut self, g: &mut Graph<N, E, Ty, Ix, Dn, De>, view: Rect)
    where
        N: Clone,
        E: Clone,
        Ty: EdgeType,
        Ix: IndexType,
        Dn: DisplayNode<N, E, Ty, Ix>,
        De: DisplayEdge<N, E, Ty, Ix, Dn>;

    /// Return current state to be stored by the layout system.
    fn state(&self) -> Self::State;
}
