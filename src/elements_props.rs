use egui::{Color32, Vec2};

pub struct NodeProps {
    pub color: Color32,
    pub position: Vec2,
    pub radius: f32,
}

impl Default for NodeProps {
    fn default() -> Self {
        Self {
            color: Color32::from_rgb(255, 255, 255),
            position: Vec2::default(),
            radius: 5.,
        }
    }
}

pub struct EdgeProps {
    pub color: Color32,
    pub width: f32,
    pub tip_size: f32,
    pub start: usize,
    pub end: usize,
    pub curve_size: f32,
}

impl EdgeProps {
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
