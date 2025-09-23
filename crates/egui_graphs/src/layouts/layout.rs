use egui::util::id_type_map::SerializableAny;
use petgraph::{stable_graph::IndexType, EdgeType};
use std::fmt::Debug;

use crate::{DisplayEdge, DisplayNode, Graph};

const KEY_PREFIX: &str = "egui_graphs_layout";

fn get_id(id: Option<String>) -> egui::Id {
    egui::Id::new(format!("{KEY_PREFIX}_{}", id.unwrap_or_default()))
}

pub trait LayoutState: SerializableAny + Default + Debug {
    fn load(ui: &egui::Ui, id: Option<String>) -> Self {
        ui.data_mut(|data| data.get_persisted::<Self>(get_id(id)).unwrap_or_default())
    }

    fn save(self, ui: &mut egui::Ui, id: Option<String>) {
        ui.data_mut(|data| {
            data.insert_persisted(get_id(id), self);
        });
    }
}

/// Optional hooks for animated/simulated layout states.
/// Implement on your layout state to allow `GraphView` helpers to force-run steps
/// even when paused and to read/write the last average displacement metric.
pub trait AnimatedState {
    fn is_running(&self) -> bool;
    fn set_running(&mut self, v: bool);

    /// Average per-node displacement from the last simulation step (graph units).
    fn last_avg_displacement(&self) -> Option<f32> {
        None
    }
    /// Store average displacement metric. Default: no-op.
    fn set_last_avg_displacement(&mut self, _v: Option<f32>) {}

    /// Retrieve current total step count (for animated/simulated layouts).
    fn step_count(&self) -> u64 {
        0
    }

    /// Set total step count.
    fn set_step_count(&mut self, _v: u64) {}
}

// Note: Step counting is part of AnimatedState for animated layouts.

pub trait Layout<S>: Default
where
    S: LayoutState,
{
    /// Creates a new layout from the given state. State is loaded and saved on every frame.
    fn from_state(state: S) -> impl Layout<S>;

    /// Called on every frame. It should update the graph layout aka nodes locations.
    fn next<N, E, Ty, Ix, Dn, De>(&mut self, g: &mut Graph<N, E, Ty, Ix, Dn, De>, ui: &egui::Ui)
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
