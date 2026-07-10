use egui_graphs::GraphChange;
use petgraph::stable_graph::IndexType;

#[derive(Clone)]
pub(crate) struct ChangeFilters {
    pub(crate) pan: bool,
    pub(crate) zoom: bool,
    pub(crate) node_move: bool,
    pub(crate) node_drag_start: bool,
    pub(crate) node_drag_end: bool,
    pub(crate) node_hover_enter: bool,
    pub(crate) node_hover_exit: bool,
    pub(crate) node_select: bool,
    pub(crate) node_deselect: bool,
    pub(crate) node_click: bool,
    pub(crate) node_double_click: bool,
    pub(crate) edge_click: bool,
    pub(crate) edge_select: bool,
    pub(crate) edge_deselect: bool,
}

impl Default for ChangeFilters {
    fn default() -> Self {
        Self {
            pan: true,
            zoom: true,
            node_move: true,
            node_drag_start: true,
            node_drag_end: true,
            node_hover_enter: true,
            node_hover_exit: true,
            node_select: true,
            node_deselect: true,
            node_click: true,
            node_double_click: true,
            edge_click: true,
            edge_select: true,
            edge_deselect: true,
        }
    }
}

impl ChangeFilters {
    pub(crate) fn enabled_for<Ix: IndexType>(&self, change: &GraphChange<Ix>) -> bool {
        match change {
            GraphChange::Panned { .. } => self.pan,
            GraphChange::Zoomed { .. } => self.zoom,
            GraphChange::NodeMoved { .. } => self.node_move,
            GraphChange::NodeDragStarted { .. } => self.node_drag_start,
            GraphChange::NodeDragEnded { .. } => self.node_drag_end,
            GraphChange::NodeHoverEntered { .. } => self.node_hover_enter,
            GraphChange::NodeHoverExited { .. } => self.node_hover_exit,
            GraphChange::NodeSelected { .. } => self.node_select,
            GraphChange::NodeDeselected { .. } => self.node_deselect,
            GraphChange::NodeClicked { .. } => self.node_click,
            GraphChange::NodeDoubleClicked { .. } => self.node_double_click,
            GraphChange::EdgeClicked { .. } => self.edge_click,
            GraphChange::EdgeSelected { .. } => self.edge_select,
            GraphChange::EdgeDeselected { .. } => self.edge_deselect,
        }
    }

    pub(crate) fn purge_disabled<Ix: IndexType>(&self, changes: &mut Vec<GraphChange<Ix>>) {
        changes.retain(|change| self.enabled_for(change));
    }
}
