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
pub struct Metadata {
    /// Whether the frame is the first one
    pub first_frame: bool,
    /// Current zoom factor
    pub zoom: f32,
    /// Current pan offset
    pub pan: Vec2,
    /// Top left position of widget
    pub top_left: Pos2,
    /// Last measured time to compute one layout step (milliseconds)
    pub last_step_time_ms: f32,
    /// Last measured time to draw the current frame, excluding the layout step (milliseconds)
    pub last_draw_time_ms: f32,
    /// Custom key to identify the metadata
    id: String,
    /// State of bounds iteration
    bounds: Bounds,
}

impl Default for Metadata {
    fn default() -> Self {
        Self {
            first_frame: true,
            zoom: 1.,
            pan: Vec2::default(),
            top_left: Pos2::default(),
            last_step_time_ms: 0.0,
            last_draw_time_ms: 0.0,
            bounds: Bounds::default(),
            id: "".to_string(),
        }
    }
}

impl Metadata {
    pub fn new(id: Option<String>) -> Self {
        Self {
            id: id.unwrap_or_default(),
            ..Default::default()
        }
    }

    pub fn load(self, ui: &egui::Ui) -> Self {
        let meta = ui.data_mut(|data| {
            data.get_persisted::<Metadata>(Id::new(self.get_key()))
                .unwrap_or(self.clone())
        });

        print!(
            "loaded metadata for key {}: {:?}\n",
            self.get_key(),
            meta.clone()
        );

        meta
    }

    pub fn save(self, ui: &mut egui::Ui) {
        ui.data_mut(|data| {
            print!(
                "saving metadata for key {}: {:?}\n",
                self.get_key(),
                self.clone()
            );

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

/// Resets [`Metadata`] state
pub fn reset_metadata(ui: &mut Ui, id: Option<String>) {
    Metadata::new(id).save(ui);
}
