use std::collections::HashMap;

use egui::Vec2;
use petgraph::stable_graph::NodeIndex;

/// `Changes` is a struct that stores the changes that happened in the graph
#[derive(Default, Debug, Clone)]
pub struct Changes {
    pub nodes: HashMap<NodeIndex, ChangesNode>,
}

impl Changes {
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

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

    pub(crate) fn select_node(&mut self, idx: NodeIndex, secondary: bool) {
        match self.nodes.get_mut(&idx) {
            Some(changes_node) => changes_node.select(secondary),
            None => {
                let mut changes_node = ChangesNode::default();
                changes_node.select(secondary);
                self.nodes.insert(idx, changes_node);
            }
        };
    }

    pub(crate) fn deselect_node(&mut self, idx: NodeIndex) {
        match self.nodes.get_mut(&idx) {
            Some(changes_node) => changes_node.deselect(),
            None => {
                let mut changes_node = ChangesNode::default();
                changes_node.deselect();
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
    pub selected_secondary: Option<bool>,
    pub dragged: Option<bool>,
    pub clicked: Option<bool>,
}

impl ChangesNode {
    fn set_location(&mut self, new_location: Vec2) {
        self.location = Some(new_location);
    }

    fn select(&mut self, secondary: bool) {
        match secondary {
            true => self.selected_secondary = Some(true),
            false => self.selected = Some(true),
        };
    }

    fn deselect(&mut self) {
        self.selected = Some(false);
        self.selected_secondary = Some(false);
    }

    fn set_dragged(&mut self, new_dragged: bool) {
        self.dragged = Some(new_dragged);
    }

    fn set_clicked(&mut self, new_clicked: bool) {
        self.clicked = Some(new_clicked);
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

        let secondary = false;
        changes.select_node(idx, secondary);
        assert_eq!(changes.nodes.get(&idx).unwrap().selected.unwrap(), true);

        let secondary = true;
        changes.select_node(idx, secondary);
        assert_eq!(
            changes.nodes.get(&idx).unwrap().selected_secondary.unwrap(),
            true
        );

        changes.select_node(idx, true);
        changes.select_node(idx, false);
        changes.deselect_node(idx);
        assert_eq!(changes.nodes.get(&idx).unwrap().selected.unwrap(), false);
        assert_eq!(
            changes.nodes.get(&idx).unwrap().selected_secondary.unwrap(),
            false
        );

        let dragged = true;
        changes.set_dragged(idx, dragged);
        assert_eq!(changes.nodes.get(&idx).unwrap().dragged.unwrap(), dragged);
    }
}
