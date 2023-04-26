#[derive(Debug, Clone)]
pub struct SettingsInteraction {
    /// Node dragging
    pub node_drag: bool,

    /// Node selection
    pub node_select: bool,

    /// Multiselection for nodes
    pub node_multiselect: bool,
}

impl Default for SettingsInteraction {
    fn default() -> Self {
        Self {
            node_drag: false,
            node_select: false,
            node_multiselect: false,
        }
    }
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
