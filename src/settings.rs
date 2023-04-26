#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct InteractionsSettings {
    /// Fit graph to the screen
    pub fit_to_screen: bool,

    /// Zoom and pan
    pub zoom_and_pan: bool,

    /// Node dragging
    pub node_drag: bool,

    /// Node selection
    pub node_select: bool,

    /// Multiselection for nodes
    pub node_multiselect: bool,
}

impl Default for InteractionsSettings {
    fn default() -> Self {
        Self {
            fit_to_screen: true,
            zoom_and_pan: false,
            node_drag: false,
            node_select: false,
            node_multiselect: false,
        }
    }
}
