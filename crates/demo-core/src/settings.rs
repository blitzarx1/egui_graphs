// Extracted demo settings types used by the demo UI

// Interaction-related toggles
pub struct SettingsInteraction {
    pub dragging_enabled: bool,
    pub hover_enabled: bool,
    pub node_clicking_enabled: bool,
    pub node_selection_enabled: bool,
    pub node_selection_multi_enabled: bool,
    pub edge_clicking_enabled: bool,
    pub edge_selection_enabled: bool,
    pub edge_selection_multi_enabled: bool,
}

impl Default for SettingsInteraction {
    fn default() -> Self {
        Self {
            dragging_enabled: true,
            hover_enabled: true,
            node_clicking_enabled: false,
            node_selection_enabled: false,
            node_selection_multi_enabled: false,
            edge_clicking_enabled: false,
            edge_selection_enabled: false,
            edge_selection_multi_enabled: false,
        }
    }
}

// Visual style toggles specific to the demo
#[derive(Default)]
pub struct SettingsStyle {
    pub labels_always: bool,
    pub edge_deemphasis: bool,
}

// Navigation & viewport parameters
pub struct SettingsNavigation {
    pub fit_to_screen_enabled: bool,
    pub zoom_and_pan_enabled: bool,
    pub zoom_speed: f32,
    pub fit_to_screen_padding: f32,
}

impl Default for SettingsNavigation {
    fn default() -> Self {
        Self {
            fit_to_screen_enabled: true,
            zoom_and_pan_enabled: false,
            zoom_speed: 0.1,
            fit_to_screen_padding: 0.1,
        }
    }
}

// Graph generation / counts controlled by UI
pub struct SettingsGraph {
    pub count_node: usize,
    pub count_edge: usize,
}

impl Default for SettingsGraph {
    fn default() -> Self {
        Self {
            count_node: 25,
            count_edge: 50,
        }
    }
}
