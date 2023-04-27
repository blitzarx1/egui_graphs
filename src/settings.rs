use egui::Color32;

#[derive(Debug, Clone, Default)]
pub struct SettingsInteraction {
    /// Node dragging
    pub node_drag: bool,

    /// Allows clicking on nodes
    pub node_click: bool,

    /// Selects clicked node, enables node_click
    pub node_select: bool,

    /// Multiselection for nodes, enables node_click
    pub node_multiselect: bool,
}

#[derive(Debug, Clone)]
pub struct SettingsNavigation {
    /// Fit graph to the screen
    pub fit_to_screen: bool,

    /// Zoom and pan
    pub zoom_and_pan: bool,

    /// Padding around the graph when fitting to screen
    pub screen_padding: f32,

    /// Zoom step
    pub zoom_step: f32,
}

impl Default for SettingsNavigation {
    fn default() -> Self {
        Self {
            screen_padding: 0.3,
            zoom_step: 0.1,
            fit_to_screen: true,
            zoom_and_pan: false,
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct SettingsStyle {
    color_node: Color32,
    color_edge: Color32,
    pub color_highlight: Color32,
    pub color_drag: Color32,
}

impl Default for SettingsStyle {
    fn default() -> Self {
        Self {
            color_node: Color32::from_rgb(200, 200, 200), // Light Gray
            color_edge: Color32::from_rgb(128, 128, 128), // Gray
            color_highlight: Color32::from_rgba_unmultiplied(100, 149, 237, 153), // Cornflower Blue
            color_drag: Color32::from_rgba_unmultiplied(240, 128, 128, 153), // Light Coral
        }
    }
}

impl SettingsStyle {
    pub fn color_node(&self, ctx: &egui::Context) -> Color32 {
        if ctx.style().visuals.dark_mode {
            return self.color_node;
        }

        self.color_edge
    }

    pub fn color_edge(&self, ctx: &egui::Context) -> Color32 {
        if ctx.style().visuals.dark_mode {
            return self.color_edge;
        }
        self.color_node
    }
}
