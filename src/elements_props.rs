use egui::{Color32};

pub struct NodeProps {
    pub color: Color32,
    pub radius: f32,
}

impl Default for NodeProps {
    fn default() -> Self {
        Self {
            color: Color32::from_rgb(255, 255, 255),
            radius: 5.,
        }
    }
}

pub struct EdgeProps {
    pub color: Color32,
    pub width: f32,
}

impl Default for EdgeProps {
    fn default() -> Self {
        Self {
            color: Color32::from_rgb(128, 128, 128),
            width: 2.,
        }
    }
}
