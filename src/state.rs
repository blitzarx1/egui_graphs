use std::collections::HashSet;

use egui::{Id, Pos2, Rect, Vec2};

#[derive(Clone)]
pub struct State {
    /// Current zoom factor
    pub zoom: f32,
    /// Current pan offset
    pub pan: Vec2,
    /// Index of the node that is currently being dragged
    node_dragged: Option<usize>,

    /// Current canvas dimensions
    pub canvas: Rect,

    /// Indices of the selected nodes
    selected_nodes: HashSet<usize>,

    /// Ids of the selected edges
    selected_edges: HashSet<(usize, usize, usize)>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            zoom: 1.,
            pan: Default::default(),
            node_dragged: Default::default(),
            canvas: Rect::from_min_max(Pos2::default(), Pos2::default()),
            selected_nodes: Default::default(),
            selected_edges: Default::default(),
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
    }

    pub fn unset_dragged_node(&mut self) {
        self.node_dragged = None;
    }

    pub fn selected_nodes(&self) -> &HashSet<usize> {
        &self.selected_nodes
    }

    pub fn selected_edges(&self) -> &HashSet<(usize, usize, usize)> {
        &self.selected_edges
    }

    pub fn select_node(&mut self, idx: usize) {
        self.selected_nodes.insert(idx);
    }

    pub fn deselect_node(&mut self, idx: usize) {
        self.selected_nodes.remove(&idx);
    }

    pub fn deselect_all_nodes(&mut self) {
        self.selected_nodes.clear();
    }

    pub fn select_edge(&mut self, idx: (usize, usize, usize)) {
        self.selected_edges.insert(idx);
    }

    pub fn deselect_edge(&mut self, idx: (usize, usize, usize)) {
        self.selected_edges.remove(&idx);
    }

    pub fn deselect_all_edges(&mut self) {
        self.selected_edges.clear();
    }
}
