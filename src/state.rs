use std::collections::HashSet;

#[derive(Default)]
pub struct State {
    dragged_node: Option<usize>,
    selected_nodes: HashSet<usize>,
    selected_edges: HashSet<(usize, usize, usize)>,
}

impl State {
    pub fn get_dragged_node(&self) -> Option<usize> {
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

    pub fn deselect_node(&mut self, idx: usize) {
        self.selected_nodes.remove(&idx);
    }

    pub fn select_edge(&mut self, idx: (usize, usize, usize)) {
        self.selected_edges.insert(idx);
    }
}
