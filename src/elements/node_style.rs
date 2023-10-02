use egui::{Color32, Context};

#[derive(Clone, Debug)]
pub struct StyleNode {
    pub radius: f32,
    color_dark: ColorsNode,
    color_light: ColorsNode,
}

impl Default for StyleNode {
    fn default() -> Self {
        Self {
            radius: 5.,
            color_dark: ColorsNode::default(),
            color_light: ColorsNode::default().inverse(),
        }
    }
}

impl StyleNode {
    pub fn color(&self, ctx: &Context) -> ColorsNode {
        if ctx.style().visuals.dark_mode {
            self.color_dark.clone()
        } else {
            self.color_light.clone()
        }
    }
}

#[derive(Clone, Debug)]
pub struct ColorsNode {
    pub main: Color32,
    pub interaction: ColorsInteractionNode,
}

impl Default for ColorsNode {
    fn default() -> Self {
        Self {
            main: Color32::from_rgb(200, 200, 200), // Light Gray
            interaction: Default::default(),
        }
    }
}

impl ColorsNode {
    fn inverse(&self) -> Self {
        Self {
            main: inverse_color(self.main),
            interaction: ColorsInteractionNode {
                selection: inverse_color(self.interaction.selection),
                drag: inverse_color(self.interaction.drag),
            },
        }
    }
}

#[derive(Clone, Debug)]
pub struct ColorsInteractionNode {
    pub selection: Color32,
    pub drag: Color32,
}

impl Default for ColorsInteractionNode {
    fn default() -> Self {
        Self {
            selection: Color32::from_rgba_unmultiplied(0, 255, 127, 153), // Spring Green
            drag: Color32::from_rgba_unmultiplied(240, 128, 128, 153),    // Light Coral
        }
    }
}

fn inverse_color(c: Color32) -> Color32 {
    Color32::from_rgb(255 - c.r(), 255 - c.g(), 255 - c.b())
}
