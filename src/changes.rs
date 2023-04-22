use std::collections::HashMap;

use egui::{Color32, Vec2};

use crate::{Edge, Node};

/// stores changes to the graph elements that are not yet applied
#[derive(Default, Clone)]
pub struct Changes {
    pub nodes: HashMap<usize, ChangesNode>,
    pub edges: HashMap<(usize, usize, usize), ChangesEdge>,
}

impl Changes {
    pub fn is_some(&self) -> bool {
        !self.nodes.is_empty() || !self.edges.is_empty()
    }

    pub fn move_node(&mut self, idx: &usize, n: &Node, delta: Vec2) {
        match self.nodes.get_mut(idx) {
            Some(changes_node) => changes_node.modify_location(n, delta),
            None => {
                let mut changes_node = ChangesNode::default();
                changes_node.modify_location(n, delta);
                self.nodes.insert(*idx, changes_node);
            }
        };
    }

    pub fn scale_node(&mut self, idx: &usize, n: &Node, factor: f32) {
        match self.nodes.get_mut(idx) {
            Some(changes_node) => changes_node.scale(n, factor),
            None => {
                let mut changes_node = ChangesNode::default();
                changes_node.scale(n, factor);
                self.nodes.insert(*idx, changes_node);
            }
        };
    }

    pub fn select_node(&mut self, idx: &usize, n: &Node) {
        match self.nodes.get_mut(idx) {
            Some(changes_node) => changes_node.select(n),
            None => {
                let mut changes_node = ChangesNode::default();
                changes_node.select(n);
                self.nodes.insert(*idx, changes_node);
            }
        };
    }

    pub fn deselect_node(&mut self, idx: &usize, n: &Node) {
        match self.nodes.get_mut(idx) {
            Some(changes_node) => changes_node.deselect(n),
            None => {
                let mut changes_node = ChangesNode::default();
                changes_node.deselect(n);
                self.nodes.insert(*idx, changes_node);
            }
        };
    }

    pub fn scale_edge(&mut self, e: &Edge, factor: f32) {
        let key = e.id();
        match self.edges.get_mut(&key) {
            Some(changes_edge) => changes_edge.scale(e, factor),
            None => {
                let mut changes_edge = ChangesEdge::default();
                changes_edge.scale(e, factor);
                self.edges.insert(key, changes_edge);
            }
        };
    }
}

/// stores deltas to the nodes properties
#[derive(Default, Clone)]
pub struct ChangesNode {
    pub location: Option<Vec2>,
    pub color: Option<Color32>,
    pub radius: Option<f32>,
    pub selected: Option<bool>,
}

impl ChangesNode {
    pub fn modify_location(&mut self, n: &Node, delta: Vec2) {
        let location = self.location.get_or_insert(n.location);
        *location += delta;
    }

    pub fn scale(&mut self, n: &Node, factor: f32) {
        let radius = self.radius.get_or_insert(n.radius);
        *radius *= factor;
    }

    fn select(&mut self, n: &Node) {
        let selected = self.selected.get_or_insert(n.selected);
        *selected = true;
    }

    fn deselect(&mut self, n: &Node) {
        let selected = self.selected.get_or_insert(n.selected);
        *selected = false;
    }
}

/// stores deltas to the edges properties
#[derive(Default, Clone)]
pub struct ChangesEdge {
    pub color: Option<Color32>,
    pub width: Option<f32>,
    pub tip_size: Option<f32>,
    pub curve_size: Option<f32>,
}

impl ChangesEdge {
    pub fn scale(&mut self, n: &Edge, factor: f32) {
        let width = self.width.get_or_insert(n.width);
        *width *= factor;

        let tip_size = self.tip_size.get_or_insert(n.tip_size);
        *tip_size *= factor;

        let curve_size = self.curve_size.get_or_insert(n.curve_size);
        *curve_size *= factor;
    }
}
