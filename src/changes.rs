use std::collections::HashMap;

use egui::Vec2;
use petgraph::stable_graph::NodeIndex;

/// `Changes` is a struct that stores the changes that happened in the graph
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_changes_default() {
        let changes: Changes = Changes::default();
        assert_eq!(changes.nodes.len(), 0);
    }

    #[test]
    fn test_changes_node_default() {
        let changes_node: ChangesNode = ChangesNode::default();
        assert!(changes_node.location.is_none());
        assert!(changes_node.selected.is_none());
        assert!(changes_node.dragged.is_none());
        assert!(changes_node.clicked.is_none());
    }

    #[test]
    fn test_setters() {
        let mut changes = Changes::default();
        let idx = NodeIndex::new(0);

        let location = Vec2::new(10.0, 10.0);
        changes.set_location(idx, location);
        assert_eq!(changes.nodes.get(&idx).unwrap().location.unwrap(), location);

        let clicked = true;
        changes.set_clicked(idx, clicked);
        assert_eq!(changes.nodes.get(&idx).unwrap().clicked.unwrap(), clicked);

        let selected = true;
        changes.set_selected(idx, selected);
        assert_eq!(changes.nodes.get(&idx).unwrap().selected.unwrap(), selected);

        let dragged = true;
        changes.set_dragged(idx, dragged);
        assert_eq!(changes.nodes.get(&idx).unwrap().dragged.unwrap(), dragged);
    }
}
