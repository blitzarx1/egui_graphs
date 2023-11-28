use egui::{Id, Pos2, Rect, Vec2};
use petgraph::{stable_graph::IndexType, EdgeType};

use crate::{DisplayNode, Node};
#[derive(Clone, Debug)]
struct BoundsIterator {
    min: Vec2,
    max: Vec2,
}

impl Default for BoundsIterator {
    fn default() -> Self {
        Self {
            min: Vec2::new(f32::MAX, f32::MAX),
            max: Vec2::new(f32::MIN, f32::MIN),
        }
    }
}

impl BoundsIterator {
    pub fn comp_iter<
        N: Clone,
        E: Clone,
        Ty: EdgeType,
        Ix: IndexType,
        D: DisplayNode<N, E, Ty, Ix>,
    >(
        &mut self,
        n: &Node<N, E, Ty, Ix, D>,
    ) {
        let loc = n.location();
        if loc.x < self.min.x {
            self.min.x = loc.x;
        };
        if loc.x > self.max.x {
            self.max.x = loc.x;
        };
        if loc.y < self.min.y {
            self.min.y = loc.y;
        };
        if loc.y > self.max.y {
            self.max.y = loc.y;
        };
    }
}

#[cfg_attr(
    feature = "egui_persistence",
    derive(serde::Serialize, serde::Deserialize)
)]
#[derive(Clone, Debug)]
pub struct Metadata {
    /// Whether the frame is the first one
    pub first_frame: bool,
    /// Current zoom factor
    pub zoom: f32,
    /// Current pan offset
    pub pan: Vec2,
    /// Top left position of widget
    pub top_left: Pos2,

    /// State of bounds iteration
    bounds_iterator: BoundsIterator,
}

impl Default for Metadata {
    fn default() -> Self {
        Self {
            first_frame: true,
            zoom: 1.,
            pan: Default::default(),
            top_left: Default::default(),
            bounds_iterator: Default::default(),
        }
    }
}

impl Metadata {
    pub fn get(ui: &egui::Ui) -> Self {
        ui.data_mut(|data| {
            data.get_persisted::<Metadata>(Id::NULL)
                .unwrap_or_default()
        })
    }

    pub fn store_into_ui(self, ui: &mut egui::Ui) {
        ui.data_mut(|data| {
            data.insert_persisted(Id::NULL, self);
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

    pub fn comp_iter_bounds<
        N: Clone,
        E: Clone,
        Ty: EdgeType,
        Ix: IndexType,
        D: DisplayNode<N, E, Ty, Ix>,
    >(
        &mut self,
        n: &Node<N, E, Ty, Ix, D>,
    ) {
        self.bounds_iterator.comp_iter(n);
    }

    /// Returns bounding rect of the graph.
    pub fn graph_bounds(&self) -> Rect {
        Rect::from_min_max(
            self.bounds_iterator.min.to_pos2(),
            self.bounds_iterator.max.to_pos2(),
        )
    }

    /// Resets the bounds iterator.
    pub fn reset_bounds_iterator(&mut self) {
        self.bounds_iterator = Default::default();
    }
}
