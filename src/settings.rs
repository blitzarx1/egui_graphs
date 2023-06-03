use egui::Color32;

use crate::{
    state_computed::{StateComputedEdge, StateComputedNode},
    Edge, Node,
};

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

    /// Allows to fold nodes.
    pub node_fold: bool,

    /// Selects clicked node, enables node_click.
    /// Select by clicking on node, deselect by clicking again.
    /// Clicking on empty space deselects all nodes.
    pub node_select: bool,

    /// How deep into the neighbours of selected nodes should the selection go.
    /// `selection_depth == 0` means only selected nodes are selected.
    /// `selection_depth > 0` means children of selected nodes are selected up to `selection_depth` generation.
    /// `selection_depth < 0` means parents of selected nodes are selected up to `selection_depth` generation.
    pub selection_depth: i32,

    /// Defines the depth up to which the children of the folded node will be folded.
    pub folding_depth: usize,

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

/// `SettingsStyle` stores settings for the style of the graph.
#[derive(Debug, Clone)]
pub struct SettingsStyle {
    color_node: Color32,
    color_edge: Color32,

    /// For every edge connected to node its radius is getting bigger by this value.
    pub edge_radius_weight: f32,

    /// Used to color children of the selected nodes.
    pub color_selection_child: Color32,

    /// Used to color parents of the selected nodes.
    pub color_selection_parent: Color32,

    /// Used to color selected nodes.
    pub color_selection: Color32,

    /// Color of nodes being dragged.
    pub color_drag: Color32,
}

impl Default for SettingsStyle {
    fn default() -> Self {
        Self {
            edge_radius_weight: 1.,
            color_selection: Color32::from_rgba_unmultiplied(0, 255, 127, 153), // Spring Green
            color_selection_child: Color32::from_rgba_unmultiplied(100, 149, 237, 153), // Cornflower Blue
            color_selection_parent: Color32::from_rgba_unmultiplied(255, 105, 180, 153), // Hot Pink
            color_node: Color32::from_rgb(200, 200, 200), // Light Gray
            color_edge: Color32::from_rgb(128, 128, 128), // Gray
            color_drag: Color32::from_rgba_unmultiplied(240, 128, 128, 153), // Light Coral
        }
    }
}

impl SettingsStyle {
    pub(crate) fn color_node<N: Clone>(&self, ctx: &egui::Context, n: &Node<N>) -> Color32 {
        if n.color.is_some() {
            return n.color.unwrap();
        }

        if ctx.style().visuals.dark_mode {
            return self.color_node;
        }

        self.color_edge
    }

    pub(crate) fn color_node_highlight<N: Clone>(
        &self,
        n: &Node<N>,
        comp: &StateComputedNode,
    ) -> Option<Color32> {
        if n.dragged {
            return Some(self.color_drag);
        }

        if n.selected {
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
        if e.color.is_some() {
            return e.color.unwrap();
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
