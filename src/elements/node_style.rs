use egui::Color32;

#[derive(Clone, Debug)]
pub struct StyleNode {
    pub radius: f32,
    pub color: StyleNodeColors,
}

impl Default for StyleNode {
    fn default() -> Self {
        Self {
            radius: 5.,
            color: Default::default(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct StyleNodeColors {
    pub main: Color32,
    pub interaction: StyleNodeInteraction,
}

impl Default for StyleNodeColors {
    fn default() -> Self {
        Self {
            main: Color32::from_rgb(200, 200, 200), // Light Gray
            interaction: Default::default(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct StyleNodeInteraction {
    pub selection: Color32,
    pub selection_child: Color32,
    pub selection_parent: Color32,
    pub drag: Color32,
}

impl Default for StyleNodeInteraction {
    fn default() -> Self {
        Self {
            selection: Color32::from_rgba_unmultiplied(0, 255, 127, 153), // Spring Green
            selection_child: Color32::from_rgba_unmultiplied(100, 149, 237, 153), // Cornflower Blue
            selection_parent: Color32::from_rgba_unmultiplied(255, 105, 180, 153), // Hot Pink
            drag: Color32::from_rgba_unmultiplied(240, 128, 128, 153),    // Light Coral
        }
    }
}
