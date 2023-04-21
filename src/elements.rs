use std::collections::HashMap;

use egui::{Color32, Vec2};

pub struct Elements {
    pub(crate) nodes: HashMap<usize, Node>,
    pub(crate) edges: HashMap<(usize, usize), Vec<Edge>>,
    /// stores the last location change in the form of a delta vector; to apply the change, add the delta to the current location;
    pub location_changes: HashMap<usize, Vec2>,
}

impl Elements {
    pub fn new(nodes: HashMap<usize, Node>, edges: HashMap<(usize, usize), Vec<Edge>>) -> Self {
        Self {
            nodes,
            edges,
            location_changes: Default::default(),
        }
    }

    pub fn get_node_mut(&mut self, idx: &usize) -> Option<&mut Node> {
        self.nodes.get_mut(idx)
    }

    pub fn get_edge_mut(&mut self, idx: &(usize, usize, usize)) -> Option<&mut Edge> {
        self.edges.get_mut(&(idx.0, idx.1))?.get_mut(idx.2)
    }
}

#[derive(Clone)]
pub struct Node {
    pub color: Color32,
    pub location: Vec2,
    pub radius: f32,
}

impl Node {
    pub fn new(location: Vec2) -> Self {
        Self {
            location,

            color: Color32::from_rgb(255, 255, 255),
            radius: 5.,
        }
    }

    pub fn location_in_screen_coords(&self, zoom: f32, pan: Vec2) -> Vec2 {
        self.location * zoom + pan
    }
}

#[derive(Clone)]
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
}
