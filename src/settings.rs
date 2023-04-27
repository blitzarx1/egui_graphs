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
