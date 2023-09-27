use egui::Color32;

#[derive(Clone, Debug)]
pub struct StyleEdge {
    pub color: StyleEdgeColors,
}

impl Default for StyleEdge {
    fn default() -> Self {
        Self {
            color: Default::default(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct StyleEdgeColors {
    pub main: Color32,
    pub interaction: StyleEdgeInteraction,
}

impl Default for StyleEdgeColors {
    fn default() -> Self {
        Self {
            main: Color32::from_rgb(200, 200, 200), // Light Gray
            interaction: Default::default(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct StyleEdgeInteraction {
    pub selection: Color32,
    pub selection_child: Color32,
    pub selection_parent: Color32,
}

impl Default for StyleEdgeInteraction {
    fn default() -> Self {
        Self {
            selection: Color32::from_rgba_unmultiplied(0, 255, 127, 153), // Spring Green
            selection_child: Color32::from_rgba_unmultiplied(100, 149, 237, 153), // Cornflower Blue
            selection_parent: Color32::from_rgba_unmultiplied(255, 105, 180, 153), // Hot Pink
        }
    }
}
