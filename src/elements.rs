use std::collections::HashMap;

use egui::{Color32, Vec2};

pub struct Elements {
    pub(crate) nodes: HashMap<usize, Node>,
    pub(crate) edges: HashMap<(usize, usize), Vec<Edge>>,
}

impl Elements {
    pub fn new(nodes: HashMap<usize, Node>, edges: HashMap<(usize, usize), Vec<Edge>>) -> Self {
        Self { nodes, edges }
    }

    pub fn get_node_mut(&mut self, idx: &usize) -> Option<&mut Node> {
        self.nodes.get_mut(idx)
    }

    pub fn get_edge_mut(&mut self, idx: &(usize, usize, usize)) -> Option<&mut Edge> {
        self.edges.get_mut(&(idx.0, idx.1))?.get_mut(idx.2)
    }
}

#[derive(Clone, Debug)]
pub struct Node {
    pub color: Color32,
    pub location: Vec2,
    pub radius: f32,
    pub selected: bool,
}

impl Node {
    pub fn new(location: Vec2) -> Self {
        Self {
            location,

            color: Color32::from_rgb(255, 255, 255),
            radius: 5.,
            selected: false,
        }
    }

    pub fn screen_transform(&self, zoom: f32, pan: Vec2) -> Self {
        Self {
            location: self.location * zoom + pan,
            radius: self.radius * zoom,

            color: self.color,
            selected: self.selected,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Edge {
    pub color: Color32,
    pub width: f32,
    pub tip_size: f32,
    pub start: usize,
    pub list_idx: usize,
    pub end: usize,
    pub curve_size: f32,
}

impl Edge {
    pub fn new(start: usize, end: usize, list_idx: usize) -> Self {
        Self {
            start,
            end,
            list_idx,

            color: Color32::from_rgb(128, 128, 128),
            width: 2.,
            tip_size: 15.,
            curve_size: 20.,
        }
    }

    pub fn id(&self) -> (usize, usize, usize) {
        (self.start, self.end, self.list_idx)
    }

    pub fn screen_transform(&self, zoom: f32) -> Self {
        Self {
            width: self.width * zoom,
            tip_size: self.tip_size * zoom,
            curve_size: self.curve_size * zoom,

            color: self.color,
            start: self.start,
            list_idx: self.list_idx,
            end: self.end,
        }
    }
}
