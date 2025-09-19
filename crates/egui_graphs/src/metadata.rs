use egui::{Id, Pos2, Rect, Ui, Vec2};
use petgraph::{stable_graph::IndexType, EdgeType};
use serde::{Deserialize, Serialize};

use crate::{node_size, DisplayNode, Node};

const KEY_PREFIX: &str = "egui_graphs_metadata";

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Bounds {
    min: Pos2,
    max: Pos2,
}

impl Default for Bounds {
    fn default() -> Self {
        Self {
            min: Pos2::new(f32::MAX, f32::MAX),
            max: Pos2::new(f32::MIN, f32::MIN),
        }
    }
}

impl Bounds {
    pub fn compute_next<
        N: Clone,
        E: Clone,
        Ty: EdgeType,
        Ix: IndexType,
        D: DisplayNode<N, E, Ty, Ix>,
    >(
        &mut self,
        n: &Node<N, E, Ty, Ix, D>,
    ) {
        let size = node_size(n, Vec2::new(0., 1.));
        let loc = n.location();

        if loc.x - size < self.min.x {
            self.min.x = loc.x - size;
        }
        if loc.x + size > self.max.x {
            self.max.x = loc.x + size;
        }
        if loc.y - size < self.min.y {
            self.min.y = loc.y - size;
        }
        if loc.y + size > self.max.y {
            self.max.y = loc.y + size;
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MetadataFrame {
    /// Current zoom factor
    pub zoom: f32,
    /// Current pan offset
    pub pan: Vec2,

    /// Last measured time to compute one layout step (milliseconds)
    pub last_step_time_ms: f32,
    /// Last measured time to draw the current frame, excluding the layout step (milliseconds)
    pub last_draw_time_ms: f32,
    /// Custom key to identify the metadata
    id: String,
    /// State of bounds iteration
    bounds: Bounds,
}

impl Default for MetadataFrame {
    fn default() -> Self {
        Self {
            zoom: 1.,
            pan: Vec2::default(),

            last_step_time_ms: 0.0,
            last_draw_time_ms: 0.0,
            bounds: Bounds::default(),
            id: "".to_string(),
        }
    }
}

impl MetadataFrame {
    pub fn new(id: Option<String>) -> Self {
        Self {
            id: id.unwrap_or_default(),
            ..Default::default()
        }
    }

    pub fn load(self, ui: &egui::Ui) -> Self {
        let meta = ui.data_mut(|data| {
            data.get_persisted::<MetadataFrame>(Id::new(self.get_key()))
                .unwrap_or(self.clone())
        });

        meta
    }

    pub fn save(self, ui: &mut egui::Ui) {
        ui.data_mut(|data| {
            data.insert_persisted(Id::new(self.get_key()), self);
        });
    }

    pub fn canvas_to_screen_pos(&self, pos: Pos2) -> Pos2 {
        (pos.to_vec2() * self.zoom + self.pan).to_pos2()
    }

    pub fn canvas_to_screen_size(&self, size: f32) -> f32 {
        size * self.zoom
    }

    pub fn screen_to_canvas_pos(&self, pos: Pos2) -> Pos2 {
        ((pos.to_vec2() - self.pan) / self.zoom).to_pos2()
    }

    pub fn process_bounds<
        N: Clone,
        E: Clone,
        Ty: EdgeType,
        Ix: IndexType,
        D: DisplayNode<N, E, Ty, Ix>,
    >(
        &mut self,
        n: &Node<N, E, Ty, Ix, D>,
    ) {
        self.bounds.compute_next(n);
    }

    /// Expands current bounds with provided rectangle (in canvas coordinates)
    pub fn expand_bounds(&mut self, min: Pos2, max: Pos2) {
        if min.x < self.bounds.min.x {
            self.bounds.min.x = min.x;
        }
        if min.y < self.bounds.min.y {
            self.bounds.min.y = min.y;
        }
        if max.x > self.bounds.max.x {
            self.bounds.max.x = max.x;
        }
        if max.y > self.bounds.max.y {
            self.bounds.max.y = max.y;
        }
    }

    /// Returns bounding rect of the graph.
    pub fn graph_bounds(&self) -> Rect {
        Rect::from_min_max(self.bounds.min, self.bounds.max)
    }

    /// Resets the bounds iterator.
    pub fn reset_bounds(&mut self) {
        self.bounds = Bounds::default();
    }

    /// Get key which is used to store metadata in egui cache.
    pub fn get_key(&self) -> String {
        format!("{KEY_PREFIX}_{}", self.id.clone())
    }
}

/// Compose an instance-scoped Id for per-widget persisted state, namespaced by custom_id.
/// Use this when storing UI-local data (like top-left or per-instance first-frame) to avoid
/// conflicts between multiple views that share the same custom_id.
pub fn instance_scoped_id(widget_id: Id, custom_id: Option<String>, suffix: &'static str) -> Id {
    widget_id.with((KEY_PREFIX, custom_id.unwrap_or_default(), suffix))
}

/// Resets [`MetadataFrame`] state
pub fn reset_metadata(ui: &mut Ui, id: Option<String>) {
    MetadataFrame::new(id).save(ui);
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MetadataInstance {
    pub last_top_left: Pos2,
    pub first_frame_pending: bool,
}

impl MetadataInstance {
    pub fn load(
        ui: &mut Ui,
        widget_id: Id,
        custom_id: &Option<String>,
        fallback_top_left: Pos2,
    ) -> Self {
        let key = instance_scoped_id(widget_id, custom_id.clone(), "local");
        ui.ctx().data_mut(|data| {
            data.get_persisted::<MetadataInstance>(key)
                .unwrap_or(MetadataInstance {
                    last_top_left: fallback_top_left,
                    first_frame_pending: true,
                })
        })
    }

    pub fn save(&self, ui: &mut Ui, widget_id: Id, custom_id: &Option<String>) {
        let key = instance_scoped_id(widget_id, custom_id.clone(), "local");
        ui.ctx().data_mut(|data| {
            data.insert_persisted(key, self.clone());
        });
    }
}

/// Compose a shared-scoped Id for values persisted per custom_id with an extra suffix.
/// Useful to store additional per-graph data alongside MetadataFrame.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MetadataSync {
    pub drag_owner: Option<String>,
    pub hover_owner: Option<String>,
}

impl MetadataSync {
    pub fn load(ui: &mut Ui, custom_id: &Option<String>) -> Self {
        let drag_owner = ui.ctx().data_mut(|data| {
            data.get_persisted::<Option<String>>(shared_instance_id(
                custom_id.clone(),
                "drag_owner",
            ))
            .unwrap_or(None)
        });
        let hover_owner = ui.ctx().data_mut(|data| {
            data.get_persisted::<Option<String>>(shared_instance_id(
                custom_id.clone(),
                "hover_owner",
            ))
            .unwrap_or(None)
        });
        Self {
            drag_owner,
            hover_owner,
        }
    }

    pub fn save(&self, ui: &mut Ui, custom_id: &Option<String>) {
        ui.ctx().data_mut(|data| {
            data.insert_persisted(
                shared_instance_id(custom_id.clone(), "drag_owner"),
                self.drag_owner.clone(),
            );
            data.insert_persisted(
                shared_instance_id(custom_id.clone(), "hover_owner"),
                self.hover_owner.clone(),
            );
        });
    }
}

pub fn shared_instance_id(custom_id: Option<String>, suffix: &'static str) -> Id {
    Id::new(format!(
        "{KEY_PREFIX}_{}_{}",
        custom_id.unwrap_or_default(),
        suffix
    ))
}

/// Compose a stable string key for instance-local maps (when you need a serializable key).
/// Uses widget_id Debug, custom_id and suffix.
pub fn instance_key_string(
    widget_id: Id,
    custom_id: Option<String>,
    suffix: &'static str,
) -> String {
    format!(
        "{KEY_PREFIX}/{}/#{:?}/{}",
        custom_id.unwrap_or_default(),
        widget_id,
        suffix
    )
}
