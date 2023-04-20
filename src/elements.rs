use std::collections::HashMap;

use egui::{Color32, Vec2};

pub struct Elements {
    pub nodes: HashMap<usize, Node>,
    pub edges: HashMap<(usize, usize), Vec<Edge>>,
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
}

#[derive(Clone)]
pub struct Edge {
    pub color: Color32,
    pub width: f32,
    pub tip_size: f32,
    pub start: usize,
    pub end: usize,
    pub curve_size: f32,
}

impl Edge {
    pub fn new(start: usize, end: usize) -> Self {
        Self {
            start,
            end,

            color: Color32::from_rgb(128, 128, 128),
            width: 2.,
            tip_size: 15.,
            curve_size: 20.,
        }
    }
}
