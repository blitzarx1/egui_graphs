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

#[derive(Default)]
pub struct SettingsInteraction {
    pub dragging_enabled: bool,
    pub node_clicking_enabled: bool,
    pub node_selection_enabled: bool,
    pub node_selection_multi_enabled: bool,
    pub edge_clicking_enabled: bool,
    pub edge_selection_enabled: bool,
    pub edge_selection_multi_enabled: bool,
}

pub struct SettingsNavigation {
    pub fit_to_screen_enabled: bool,
    pub zoom_and_pan_enabled: bool,
    pub zoom_speed: f32,
}

impl Default for SettingsNavigation {
    fn default() -> Self {
        Self {
            zoom_speed: 0.1,
            fit_to_screen_enabled: true,
            zoom_and_pan_enabled: false,
        }
    }
}

#[derive(Default)]
pub struct SettingsStyle {
    pub labels_always: bool,
}

pub struct SettingsSimulation {
    pub dt: f32,
    pub cooloff_factor: f32,
    pub scale: f32,
}

impl Default for SettingsSimulation {
    fn default() -> Self {
        Self {
            dt: 0.03,
            cooloff_factor: 0.85,
            scale: 100.,
        }
    }
}
