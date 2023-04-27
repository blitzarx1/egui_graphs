use std::collections::HashMap;

use egui::Vec2;

use crate::{Edge, Node};

/// Stores changes to the graph elements that are not yet applied
#[derive(Default, Clone)]
pub struct Changes {
    pub(crate) nodes: HashMap<usize, ChangesNode>,
    pub(crate) edges: HashMap<(usize, usize, usize), ChangesEdge>,
}

impl Changes {
    pub(crate) fn move_node(&mut self, idx: &usize, n: &Node, delta: Vec2) {
        match self.nodes.get_mut(idx) {
            Some(changes_node) => changes_node.modify_location(n, delta),
            None => {
                let mut changes_node = ChangesNode::default();
                changes_node.modify_location(n, delta);
                self.nodes.insert(*idx, changes_node);
            }
        };
    }

    pub(crate) fn click_node(&mut self, idx: &usize) {
        match self.nodes.get_mut(idx) {
            Some(changes_node) => changes_node.click(),
            None => {
                let mut changes_node = ChangesNode::default();
                changes_node.click();
                self.nodes.insert(*idx, changes_node);
            }
        };
    }

    pub(crate) fn select_node(&mut self, idx: &usize, n: &Node) {
        match self.nodes.get_mut(idx) {
            Some(changes_node) => changes_node.select(n),
            None => {
                let mut changes_node = ChangesNode::default();
                changes_node.select(n);
                self.nodes.insert(*idx, changes_node);
            }
        };
    }

    pub(crate) fn set_dragged_node(&mut self, idx: &usize, n: &Node) {
        match self.nodes.get_mut(idx) {
            Some(changes_node) => changes_node.set_drag(n),
            None => {
                let mut changes_node = ChangesNode::default();
                changes_node.set_drag(n);
                self.nodes.insert(*idx, changes_node);
            }
        };
    }

    pub(crate) fn unset_dragged_node(&mut self, idx: &usize, n: &Node) {
        match self.nodes.get_mut(idx) {
            Some(changes_node) => changes_node.unset_drag(n),
            None => {
                let mut changes_node = ChangesNode::default();
                changes_node.unset_drag(n);
                self.nodes.insert(*idx, changes_node);
            }
        };
    }

    pub(crate) fn deselect_node(&mut self, idx: &usize, n: &Node) {
        match self.nodes.get_mut(idx) {
            Some(changes_node) => changes_node.deselect(n),
            None => {
                let mut changes_node = ChangesNode::default();
                changes_node.deselect(n);
                self.nodes.insert(*idx, changes_node);
            }
        };
    }

    pub(crate) fn select_edge(&mut self, idx: &(usize, usize, usize), e: &Edge) {
        match self.edges.get_mut(idx) {
            Some(changes_edge) => changes_edge.select(e),
            None => {
                let mut changes_edge = ChangesEdge::default();
                changes_edge.select(e);
                self.edges.insert(*idx, changes_edge);
            }
        };
    }

    pub(crate) fn deselect_edge(&mut self, idx: &(usize, usize, usize), e: &Edge) {
        match self.edges.get_mut(idx) {
            Some(changes_edge) => changes_edge.deselect(e),
            None => {
                let mut changes_edge = ChangesEdge::default();
                changes_edge.deselect(e);
                self.edges.insert(*idx, changes_edge);
            }
        };
    }
}

/// Stores changes to the node properties
#[derive(Default, Clone)]
pub struct ChangesNode {
    pub location: Option<Vec2>,
    pub radius: Option<f32>,
    pub selected: Option<bool>,
    pub dragged: Option<bool>,
    pub clicked: Option<bool>,
}

impl ChangesNode {
    fn modify_location(&mut self, n: &Node, delta: Vec2) {
        let location = self.location.get_or_insert(n.location);
        *location += delta;
    }

    fn select(&mut self, n: &Node) {
        let selected = self.selected.get_or_insert(n.selected);
        *selected = true;
    }

    fn deselect(&mut self, n: &Node) {
        let selected = self.selected.get_or_insert(n.selected);
        *selected = false;
    }

    fn set_drag(&mut self, n: &Node) {
        let dragged = self.dragged.get_or_insert(n.dragged);
        *dragged = true;
    }

    fn unset_drag(&mut self, n: &Node) {
        let dragged = self.dragged.get_or_insert(n.dragged);
        *dragged = false;
    }

    fn click(&mut self) {
        let clicked = self.clicked.get_or_insert(true);
        *clicked = true;
    }
}

/// Stores changes to the edge properties
#[derive(Default, Clone)]
pub struct ChangesEdge {
    pub selected: Option<bool>,
}

impl ChangesEdge {
    fn select(&mut self, n: &Edge) {
        let selected = self.selected.get_or_insert(n.selected);
        *selected = true;
    }

    fn deselect(&mut self, n: &Edge) {
        let selected = self.selected.get_or_insert(n.selected);
        *selected = false;
    }
}
