use egui::util::id_type_map::SerializableAny;
use petgraph::{stable_graph::IndexType, EdgeType};
use std::fmt::Debug;

use crate::{DisplayEdge, DisplayNode, Graph};

const KEY_PREFIX: &str = "egui_graphs_layout";
fn get_key(key: String) -> String {
    format!("{KEY_PREFIX}_{key}")
}

pub trait LayoutState: SerializableAny + Default + Debug {
    fn load(ui: &egui::Ui, key: String) -> Self {
        ui.data_mut(|data| {
            data.get_persisted::<Self>(egui::Id::new(get_key(key)))
                .unwrap_or_default()
        })
    }

    fn save(self, ui: &mut egui::Ui, key: String) {
        ui.data_mut(|data| {
            data.insert_persisted(egui::Id::new(get_key(key)), self);
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
    /// Default: None (not provided by the layout).
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
    /// Convenience: increment step count (saturating add).
    fn inc_step_count(&mut self) {
        let n = self.step_count();
        self.set_step_count(n.saturating_add(1));
    }
    /// Convenience: reset step count to zero.
    fn reset_step_count(&mut self) {
        self.set_step_count(0);
    }
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
