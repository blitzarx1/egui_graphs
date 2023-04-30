use egui::Color32;

/// `SettingsInteraction` stores settings for the interaction with the graph.
///
/// `node_click` is included in `node_select` and `node_multiselect`.
/// `node_select` is included in `node_multiselect`.
#[derive(Debug, Clone, Default)]
pub struct SettingsInteraction {
    /// Node dragging. To drag a node, click, hold and drag.
    pub node_drag: bool,

    /// Allows clicking on nodes.
    pub node_click: bool,

    /// Selects clicked node, enables node_click.
    /// Select by clicking on node, deselect by clicking again.
    /// Clicking on empty space deselects all nodes.
    pub node_select: bool,

    /// Multiselection for nodes, enables node_click and node_select.
    pub node_multiselect: bool,
}

/// `SettingsNavigation` stores settings for the navigation around the graph.
#[derive(Debug, Clone)]
pub struct SettingsNavigation {
    /// Fit graph to the screen. With this enabled, the graph will be scaled to
    /// fit the screen on every frate.
    ///
    /// You can cofigure the padding around the graph with `screen_padding`.
    pub fit_to_screen: bool,

    /// Padding around the graph when fitting to screen.
    pub screen_padding: f32,

    /// Zoom and pan. Zoom with ctrl + mouse wheel, pan with mouse drag.
    pub zoom_and_pan: bool,

    /// Zoom step.
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
pub struct SettingsStyle {
    color_node: Color32,
    color_edge: Color32,

    pub edge_radius_weight: f32,

    pub color_highlight: Color32,
    pub color_drag: Color32,
}

impl Default for SettingsStyle {
    fn default() -> Self {
        Self {
            edge_radius_weight: 1.,
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
