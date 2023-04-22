use std::collections::HashSet;

use egui::{Id, Pos2, Rect, Vec2};

#[derive(Clone)]
pub struct State {
    /// current zoom factor
    pub zoom: f32,
    /// current pan offset
    pub pan: Vec2,
    /// index of the node that is currently being dragged
    node_dragged: Option<usize>,

    // nodes which neede selection drawing
    pub nodes_selected: HashSet<usize>,

    /// current canvas dimensions
    pub canvas: Rect,
}

impl Default for State {
    fn default() -> Self {
        Self {
            zoom: 1.,
            pan: Default::default(),
            node_dragged: Default::default(),
            nodes_selected: Default::default(),
            canvas: Rect::from_min_max(Pos2::default(), Pos2::default()),
        }
    }
}

impl State {
    pub fn get(ui: &mut egui::Ui) -> Self {
        ui.data_mut(|data| data.get_persisted::<State>(Id::null()).unwrap_or_default())
    }

    pub fn store(self, ui: &mut egui::Ui) {
        ui.data_mut(|data| {
            data.insert_persisted(Id::null(), self);
        });
    }

    pub fn get_dragged_node(&self) -> Option<usize> {
        self.node_dragged
    }

    pub fn set_dragged_node(&mut self, idx: usize) {
        self.node_dragged = Some(idx);
        self.select_node(idx);
    }

    pub fn unset_dragged_node(&mut self) {
        self.deselect_node(self.node_dragged.unwrap());
        self.node_dragged = None;
    }

    pub fn select_node(&mut self, idx: usize) {
        self.nodes_selected.insert(idx);
    }

    pub fn deselect_node(&mut self, idx: usize) {
        self.nodes_selected.remove(&idx);
    }
}
