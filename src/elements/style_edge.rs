use egui::Color32;

#[derive(Clone, Debug)]
pub struct StyleEdge {
    pub width: f32,
    pub tip_size: f32,
    pub tip_angle: f32,
    pub curve_size: f32,
    pub color: StyleEdgeColors,
}

impl Default for StyleEdge {
    fn default() -> Self {
        Self {
            width: 2.,
            tip_size: 15.,
            tip_angle: std::f32::consts::TAU / 30.,
            curve_size: 20.,

            color: Default::default(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct StyleEdgeColors {
    pub main: Color32,
}

impl Default for StyleEdgeColors {
    fn default() -> Self {
        Self {
            main: Color32::from_rgb(128, 128, 128), // Gray
        }
    }
}
