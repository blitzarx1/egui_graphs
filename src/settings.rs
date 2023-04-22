#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Settings {
    /// fits graph to the screen
    pub fit_to_screen: bool,

    /// enables/disables node dragging
    pub node_drag: bool,

    /// enables/disables node selection
    pub node_select: bool,

    /// enables/disables zoom and pan
    pub zoom_and_pan: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            fit_to_screen: true,
            node_drag: false,
            zoom_and_pan: false,
            node_select: false,
        }
    }
}
