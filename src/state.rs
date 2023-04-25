use std::collections::HashSet;

/// `FrameState` stores the state of the graph elements
/// in the current frame. It is created and destroyed by the widget
/// on every frame.
#[derive(Default)]
pub struct FrameState {
    dragged_node: Option<usize>,
    selected_nodes: HashSet<usize>,
    selected_edges: HashSet<(usize, usize, usize)>,
}

impl FrameState {
    pub fn dragged_node(&self) -> Option<usize> {
        self.dragged_node
    }

    pub fn set_dragged_node(&mut self, idx: usize) {
        self.dragged_node = Some(idx);
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

    pub fn select_edge(&mut self, idx: (usize, usize, usize)) {
        self.selected_edges.insert(idx);
    }
}
