use egui::Color32;

use crate::{
    state_computed::{StateComputedEdge, StateComputedNode},
    Edge, Node,
};

/// Represents graph interaction settings.
#[derive(Debug, Clone, Default)]
pub struct SettingsInteraction {
    pub(crate) dragging_enabled: bool,
    pub(crate) clicking_enabled: bool,
    pub(crate) folding_enabled: bool,
    pub(crate) selection_enabled: bool,
    pub(crate) selection_multi_enabled: bool,
    pub(crate) selection_depth: i32,
    pub(crate) folding_depth: usize,
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
    pub fn with_clicking_enabled(mut self, enabled: bool) -> Self {
        self.clicking_enabled = enabled;
        self
    }

    /// Allows to fold nodes.
    ///
    /// Default: `false`
    pub fn with_folding_enabled(mut self, enabled: bool) -> Self {
        self.folding_enabled = enabled;
        self
    }

    /// Selects clicked node, enables clicks.
    ///
    /// Select by clicking on node, deselect by clicking again.
    ///
    /// Clicking on empty space deselects all nodes.
    ///
    /// Default: `false`
    pub fn with_selection_enabled(mut self, enabled: bool) -> Self {
        self.selection_enabled = enabled;
        self
    }

    /// Multiselection for nodes, enables click and select.
    ///
    /// Default: `false`
    pub fn with_selection_multi_enabled(mut self, enabled: bool) -> Self {
        self.selection_multi_enabled = enabled;
        self
    }

    /// How deep into the neighbours of selected nodes should the selection go.
    ///
    /// * `selection_depth == 0` means only selected nodes are selected.
    /// * `selection_depth > 0` means children of selected nodes are selected up to `selection_depth` generation.
    /// * `selection_depth < 0` means parents of selected nodes are selected up to `selection_depth` generation.
    /// * passing `i32::MAX` and `i32::MIN` selects all available generations of children or parents.
    ///
    /// Default: `0`
    pub fn with_selection_depth(mut self, depth: i32) -> Self {
        self.selection_depth = depth;
        self
    }

    /// Defines the generation depth up to which the children of the folded node will be folded.
    ///
    /// * `folding_depth == 0` means only the folded node is folded.
    /// * `folding_depth > 0` means children of the folded node are folded up to `folding_depth` generation.
    /// * `folding_depth == usize::MAX` folds all available generations of children.
    ///
    /// Default: `0`
    pub fn with_folding_depth(mut self, depth: usize) -> Self {
        self.folding_depth = depth;
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
            fit_to_screen_enabled: true,
            zoom_and_pan_enabled: false,
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
#[derive(Debug, Clone)]
pub struct SettingsStyle {
    pub(crate) labels_always: bool,
    pub(crate) edge_radius_weight: f32,
    pub(crate) folded_radius_weight: f32,

    /// Used to color children of the selected nodes.
    pub(crate) color_selection_child: Color32,

    /// Used to color parents of the selected nodes.
    pub(crate) color_selection_parent: Color32,

    /// Used to color selected nodes.
    pub(crate) color_selection: Color32,

    /// Color of nodes being dragged.
    pub(crate) color_drag: Color32,

    /// Text color for light background.
    pub(crate) color_text_light: Color32,

    /// Text color for dark background.
    pub(crate) color_text_dark: Color32,

    color_node: Color32,
    color_edge: Color32,
}

impl Default for SettingsStyle {
    fn default() -> Self {
        Self {
            edge_radius_weight: 1.,
            folded_radius_weight: 2.,
            color_selection: Color32::from_rgba_unmultiplied(0, 255, 127, 153), // Spring Green
            color_selection_child: Color32::from_rgba_unmultiplied(100, 149, 237, 153), // Cornflower Blue
            color_selection_parent: Color32::from_rgba_unmultiplied(255, 105, 180, 153), // Hot Pink
            color_node: Color32::from_rgb(200, 200, 200), // Light Gray
            color_edge: Color32::from_rgb(128, 128, 128), // Gray
            color_drag: Color32::from_rgba_unmultiplied(240, 128, 128, 153), // Light Coral
            color_text_light: Color32::WHITE,
            color_text_dark: Color32::BLACK,
            labels_always: Default::default(),
        }
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

    /// For every node folded the folding root node radius is getting bigger by this value.
    ///
    /// Default: `2.`
    pub fn with_folded_radius_weight(mut self, weight: f32) -> Self {
        self.folded_radius_weight = weight;
        self
    }

    /// For every edge connected to node its radius is getting bigger by this value.
    ///
    /// Default: `1.`
    pub fn with_edge_radius_weight(mut self, weight: f32) -> Self {
        self.edge_radius_weight = weight;
        self
    }

    pub(crate) fn color_node<N: Clone>(&self, ctx: &egui::Context, n: &Node<N>) -> Color32 {
        if n.color().is_some() {
            return n.color().unwrap();
        }

        if ctx.style().visuals.dark_mode {
            return self.color_node;
        }

        self.color_edge
    }

    pub(crate) fn color_label(&self, ctx: &egui::Context) -> Color32 {
        match ctx.style().visuals.dark_mode {
            true => self.color_text_light,
            false => self.color_text_dark,
        }
    }

    pub(crate) fn color_node_highlight<N: Clone>(
        &self,
        n: &Node<N>,
        comp: &StateComputedNode,
    ) -> Option<Color32> {
        if n.dragged() {
            return Some(self.color_drag);
        }

        if n.selected() {
            return Some(self.color_selection);
        }

        if comp.selected_child {
            return Some(self.color_selection_child);
        }

        if comp.selected_parent {
            return Some(self.color_selection_parent);
        }

        None
    }

    pub(crate) fn color_edge<E: Clone>(&self, ctx: &egui::Context, e: &Edge<E>) -> Color32 {
        if e.color().is_some() {
            return e.color().unwrap();
        }

        if ctx.style().visuals.dark_mode {
            return self.color_edge;
        }

        self.color_node
    }

    pub(crate) fn color_edge_highlight(&self, comp: &StateComputedEdge) -> Option<Color32> {
        if comp.selected_child {
            return Some(self.color_selection_child);
        }

        if comp.selected_parent {
            return Some(self.color_selection_parent);
        }

        None
    }
}
