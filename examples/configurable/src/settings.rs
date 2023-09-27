pub struct SettingsGraph {
    pub count_node: usize,
    pub count_edge: usize,
}

impl Default for SettingsGraph {
    fn default() -> Self {
        Self {
            count_node: 300,
            count_edge: 500,
        }
    }
}

#[derive(Default)]
pub struct SettingsInteraction {
    pub dragging_enabled: bool,
    pub clicking_enabled: bool,
    pub selection_enabled: bool,
    pub selection_multi_enabled: bool,
}

pub struct SettingsNavigation {
    pub fit_to_screen_enabled: bool,
    pub zoom_and_pan_enabled: bool,
    pub screen_padding: f32,
    pub zoom_speed: f32,
}

impl Default for SettingsNavigation {
    fn default() -> Self {
        Self {
            screen_padding: 0.3,
            zoom_speed: 0.1,
            fit_to_screen_enabled: true,
            zoom_and_pan_enabled: false,
        }
    }
}

pub struct SettingsStyle {
    pub edge_radius_weight: f32,
    pub labels_always: bool,
}

impl Default for SettingsStyle {
    fn default() -> Self {
        Self {
            edge_radius_weight: 1.,
            labels_always: false,
        }
    }
}
