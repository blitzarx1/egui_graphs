use std::collections::HashMap;

use egui::Vec2;
use petgraph::stable_graph::NodeIndex;

/// Stores changes to the graph elements that are not yet applied.
/// Currently stores changes only to the nodes as there are no
/// actions which can be applied to the edges tracked by the GraphView widget.
#[derive(Default, Debug, Clone)]
pub struct Changes {
    pub(crate) nodes: HashMap<NodeIndex, ChangesNode>,
}

impl Changes {
    pub(crate) fn set_location(&mut self, idx: NodeIndex, val: Vec2) {
        match self.nodes.get_mut(&idx) {
            Some(changes_node) => changes_node.set_location(val),
            None => {
                let mut changes_node = ChangesNode::default();
                changes_node.set_location(val);
                self.nodes.insert(idx, changes_node);
            }
        };
    }

    pub(crate) fn set_clicked(&mut self, idx: NodeIndex, val: bool) {
        match self.nodes.get_mut(&idx) {
            Some(changes_node) => changes_node.set_clicked(val),
            None => {
                let mut changes_node = ChangesNode::default();
                changes_node.set_clicked(val);
                self.nodes.insert(idx, changes_node);
            }
        };
    }

    pub(crate) fn set_selected(&mut self, idx: NodeIndex, val: bool) {
        match self.nodes.get_mut(&idx) {
            Some(changes_node) => changes_node.set_selected(val),
            None => {
                let mut changes_node = ChangesNode::default();
                changes_node.set_selected(val);
                self.nodes.insert(idx, changes_node);
            }
        };
    }

    pub(crate) fn set_dragged(&mut self, idx: NodeIndex, val: bool) {
        match self.nodes.get_mut(&idx) {
            Some(changes_node) => changes_node.set_dragged(val),
            None => {
                let mut changes_node = ChangesNode::default();
                changes_node.set_dragged(val);
                self.nodes.insert(idx, changes_node);
            }
        };
    }
}

/// Stores changes to the node properties
#[derive(Default, Debug, Clone)]
pub struct ChangesNode {
    pub location: Option<Vec2>,
    pub selected: Option<bool>,
    pub dragged: Option<bool>,
    pub clicked: Option<bool>,
}

impl ChangesNode {
    fn set_location(&mut self, new_location: Vec2) {
        self.location.get_or_insert(new_location);
    }

    fn set_selected(&mut self, new_selected: bool) {
        self.selected.get_or_insert(new_selected);
    }

    fn set_dragged(&mut self, new_dragged: bool) {
        self.dragged.get_or_insert(new_dragged);
    }

    fn set_clicked(&mut self, new_clicked: bool) {
        self.clicked.get_or_insert(new_clicked);
    }
}
