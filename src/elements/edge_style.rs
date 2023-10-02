use egui::{Color32, Context};

use super::color;

#[derive(Clone, Debug)]
pub struct StyleEdge {
    pub width: f32,
    pub tip_size: f32,
    pub tip_angle: f32,
    pub curve_size: f32,
    color_dark: ColorsEdge,
    color_light: ColorsEdge,
}

impl Default for StyleEdge {
    fn default() -> Self {
        Self {
            width: 2.,
            tip_size: 15.,
            tip_angle: std::f32::consts::TAU / 30.,
            curve_size: 20.,

            color_dark: ColorsEdge::default(),
            color_light: ColorsEdge::default().inverse(),
        }
    }
}

impl StyleEdge {
    pub fn color(&self, ctx: &Context) -> ColorsEdge {
        if ctx.style().visuals.dark_mode {
            self.color_dark.clone()
        } else {
            self.color_light.clone()
        }
    }
}

#[derive(Clone, Debug)]
pub struct ColorsEdge {
    pub main: Color32,
}

impl Default for ColorsEdge {
    fn default() -> Self {
        Self {
            main: Color32::from_rgb(128, 128, 128), // Gray
        }
    }
}

impl ColorsEdge {
    fn inverse(&self) -> Self {
        Self {
            main: color::inverse(self.main),
        }
    }
}
