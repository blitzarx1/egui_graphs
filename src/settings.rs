/// Represents graph interaction settings.
#[derive(Debug, Clone)]
pub struct SettingsInteraction {
    pub(crate) dragging_enabled: bool,
    pub(crate) node_clicking_enabled: bool,
    pub(crate) node_selection_enabled: bool,
    pub(crate) node_selection_multi_enabled: bool,
    pub(crate) edge_clicking_enabled: bool,
    pub(crate) edge_selection_enabled: bool,
    pub(crate) edge_selection_multi_enabled: bool,
}

impl Default for SettingsInteraction {
    fn default() -> Self {
        Self {
            dragging_enabled: true,
            node_clicking_enabled: false,
            node_selection_enabled: false,
            node_selection_multi_enabled: false,
            edge_clicking_enabled: false,
            edge_selection_enabled: false,
            edge_selection_multi_enabled: false,
        }
    }
}

impl SettingsInteraction {
    /// Creates new [`SettingsInteraction`] with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Node dragging. To drag a node with your mouse or finger.
    ///
    /// Default: `false`
    pub fn with_dragging_enabled(mut self, enabled: bool) -> Self {
        self.dragging_enabled = enabled;
        self
    }

    /// Allows clicking on nodes.
    ///
    /// Default: `false`
    pub fn with_node_clicking_enabled(mut self, enabled: bool) -> Self {
        self.node_clicking_enabled = enabled;
        self
    }

    /// Selects clicked node, enables clicks.
    ///
    /// Select by clicking on node, deselect by clicking again.
    ///
    /// Clicking on empty space deselects all nodes.
    ///
    /// Default: `false`
    pub fn with_node_selection_enabled(mut self, enabled: bool) -> Self {
        self.node_selection_enabled = enabled;
        self
    }

    /// Multiselection for nodes, enables click and select.
    ///
    /// Default: `false`
    pub fn with_node_selection_multi_enabled(mut self, enabled: bool) -> Self {
        self.node_selection_multi_enabled = enabled;
        self
    }

    /// Allows clicking on edges.
    ///
    /// Default: `false`
    pub fn with_edge_clicking_enabled(mut self, enabled: bool) -> Self {
        self.edge_clicking_enabled = enabled;
        self
    }

    /// Selects clicked edge, enables clicks.
    ///
    /// Select by clicking on a edge, deselect by clicking again.
    ///
    /// Clicking on empty space deselects all edges.
    ///
    /// Default: `false`
    pub fn with_edge_selection_enabled(mut self, enabled: bool) -> Self {
        self.edge_selection_enabled = enabled;
        self
    }

    /// Multiselection for edges, enables click and select.
    ///
    /// Default: `false`
    pub fn with_edge_selection_multi_enabled(mut self, enabled: bool) -> Self {
        self.edge_selection_multi_enabled = enabled;
        self
    }
}

/// Represents graph navigation settings.
#[derive(Debug, Clone)]
pub struct SettingsNavigation {
    pub(crate) fit_to_screen_enabled: bool,
    pub(crate) zoom_and_pan_enabled: bool,
    pub(crate) screen_padding: f32,
    pub(crate) zoom_speed: f32,
}

impl Default for SettingsNavigation {
    fn default() -> Self {
        Self {
            screen_padding: 0.3,
            zoom_speed: 0.1,
            fit_to_screen_enabled: false,
            zoom_and_pan_enabled: true,
        }
    }
}

impl SettingsNavigation {
    /// Creates new [`SettingsNavigation`] with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Fits the graph to the screen.
    ///
    /// With this enabled, the graph will be scaled and panned to fit the screen on every frame.
    ///
    /// You can configure the padding around the graph with `screen_padding` setting.
    ///
    /// Default: `true`
    pub fn with_fit_to_screen_enabled(mut self, enabled: bool) -> Self {
        self.fit_to_screen_enabled = enabled;
        self
    }

    /// Zoom with ctrl + mouse wheel, pan with mouse drag.
    ///
    /// Default: `false`
    pub fn with_zoom_and_pan_enabled(mut self, enabled: bool) -> Self {
        self.zoom_and_pan_enabled = enabled;
        self
    }

    /// Padding around the graph when fitting to the screen.
    pub fn with_screen_padding(mut self, padding: f32) -> Self {
        self.screen_padding = padding;
        self
    }

    /// Controls the speed of the zoom.
    pub fn with_zoom_speed(mut self, speed: f32) -> Self {
        self.zoom_speed = speed;
        self
    }
}

/// `SettingsStyle` stores settings for the style of the graph.
#[derive(Clone, Default)]
pub struct SettingsStyle {
    pub(crate) labels_always: bool,
    // Optional user-provided hook to override node stroke (outline) styling.
    // Signature: (selected, dragged, node_color, current_stroke, egui_style) -> new Stroke
    pub(crate) node_stroke_hook: Option<
        std::sync::Arc<
            dyn Fn(bool, bool, Option<egui::Color32>, egui::Stroke, &egui::Style) -> egui::Stroke
                + Send
                + Sync,
        >,
    >,
    // Optional user-provided hook to override edge stroke styling.
    // Signature: (selected, order, current_stroke, egui_style) -> new Stroke
    pub(crate) edge_stroke_hook: Option<
        std::sync::Arc<
            dyn Fn(bool, usize, egui::Stroke, &egui::Style) -> egui::Stroke + Send + Sync,
        >,
    >,
}

impl core::fmt::Debug for SettingsStyle {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("SettingsStyle")
            .field("labels_always", &self.labels_always)
            .field(
                "node_stroke_hook",
                &self.node_stroke_hook.as_ref().map(|_| "<hook>"),
            )
            .field(
                "edge_stroke_hook",
                &self.edge_stroke_hook.as_ref().map(|_| "<hook>"),
            )
            .finish()
    }
}

impl SettingsStyle {
    /// Creates new [`SettingsStyle`] with default values.
    /// ```
    /// use egui_graphs::SettingsStyle;
    /// let settings = SettingsStyle::new();
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// Whether to show labels always or when interacted.
    ///
    /// Default is false.
    pub fn with_labels_always(mut self, always: bool) -> Self {
        self.labels_always = always;
        self
    }

    /// Provide a hook to customize node stroke (outline) styling.
    /// The hook receives: (selected, dragged, node_color, current_stroke, egui_style) and should return a new Stroke.
    /// Example:
    /// ```
    /// use egui_graphs::SettingsStyle;
    /// use egui::{Stroke, Color32};
    /// let style = SettingsStyle::new().with_node_stroke_hook(|selected, _dragged, color, stroke, egui_style| {
    ///     let mut s = stroke;
    ///     // Base color from node or egui visuals
    ///     s.color = color.unwrap_or(egui_style.visuals.widgets.inactive.fg_stroke.color);
    ///     if selected { s.width = 3.0; }
    ///     s
    /// });
    /// ```
    pub fn with_node_stroke_hook<F>(mut self, f: F) -> Self
    where
        F: Fn(bool, bool, Option<egui::Color32>, egui::Stroke, &egui::Style) -> egui::Stroke
            + Send
            + Sync
            + 'static,
    {
        self.node_stroke_hook = Some(std::sync::Arc::new(f));
        self
    }

    /// Provide a hook to customize edge stroke styling.
    /// The hook receives: (selected, order, current_stroke, egui_style) and should return a new Stroke.
    pub fn with_edge_stroke_hook<F>(mut self, f: F) -> Self
    where
        F: Fn(bool, usize, egui::Stroke, &egui::Style) -> egui::Stroke + Send + Sync + 'static,
    {
        self.edge_stroke_hook = Some(std::sync::Arc::new(f));
        self
    }
}
