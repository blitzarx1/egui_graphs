use std::collections::HashMap;

use egui::Vec2;

use crate::Node;

/// Stores changes to the graph elements that are not yet applied.
/// Currently stores changes only to the nodes as there are no
/// actions which can be applied to the edges tracked by the GraphView widget.
#[derive(Default, Clone)]
pub struct Changes {
    pub(crate) nodes: HashMap<usize, ChangesNode>,
}

impl Changes {
    pub(crate) fn move_node(&mut self, idx: &usize, n: &Node, delta: Vec2) {
        match self.nodes.get_mut(idx) {
            Some(changes_node) => changes_node.modify_location(n, delta),
            None => {
                let mut changes_node = ChangesNode::default();
                changes_node.modify_location(n, delta);
                self.nodes.insert(*idx, changes_node);
            }
        };
    }

    pub(crate) fn click_node(&mut self, idx: &usize) {
        match self.nodes.get_mut(idx) {
            Some(changes_node) => changes_node.click(),
            None => {
                let mut changes_node = ChangesNode::default();
                changes_node.click();
                self.nodes.insert(*idx, changes_node);
            }
        };
    }

    pub(crate) fn select_node(&mut self, idx: &usize, n: &Node) {
        match self.nodes.get_mut(idx) {
            Some(changes_node) => changes_node.select(n),
            None => {
                let mut changes_node = ChangesNode::default();
                changes_node.select(n);
                self.nodes.insert(*idx, changes_node);
            }
        };
    }

    pub(crate) fn set_dragged_node(&mut self, idx: &usize, n: &Node) {
        match self.nodes.get_mut(idx) {
            Some(changes_node) => changes_node.set_drag(n),
            None => {
                let mut changes_node = ChangesNode::default();
                changes_node.set_drag(n);
                self.nodes.insert(*idx, changes_node);
            }
        };
    }

    pub(crate) fn unset_dragged_node(&mut self, idx: &usize, n: &Node) {
        match self.nodes.get_mut(idx) {
            Some(changes_node) => changes_node.unset_drag(n),
            None => {
                let mut changes_node = ChangesNode::default();
                changes_node.unset_drag(n);
                self.nodes.insert(*idx, changes_node);
            }
        };
    }

    pub(crate) fn deselect_node(&mut self, idx: &usize, n: &Node) {
        match self.nodes.get_mut(idx) {
            Some(changes_node) => changes_node.deselect(n),
            None => {
                let mut changes_node = ChangesNode::default();
                changes_node.deselect(n);
                self.nodes.insert(*idx, changes_node);
            }
        };
    }
}

/// Stores changes to the node properties
#[derive(Default, Clone)]
pub struct ChangesNode {
    pub location: Option<Vec2>,
    pub radius: Option<f32>,
    pub selected: Option<bool>,
    pub dragged: Option<bool>,
    pub clicked: Option<bool>,
}

impl ChangesNode {
    fn modify_location(&mut self, n: &Node, delta: Vec2) {
        let location = self.location.get_or_insert(n.location);
        *location += delta;
    }

    fn select(&mut self, n: &Node) {
        let selected = self.selected.get_or_insert(n.selected);
        *selected = true;
    }

    fn deselect(&mut self, n: &Node) {
        let selected = self.selected.get_or_insert(n.selected);
        *selected = false;
    }

    fn set_drag(&mut self, n: &Node) {
        let dragged = self.dragged.get_or_insert(n.dragged);
        *dragged = true;
    }

    fn unset_drag(&mut self, n: &Node) {
        let dragged = self.dragged.get_or_insert(n.dragged);
        *dragged = false;
    }

    fn click(&mut self) {
        let clicked = self.clicked.get_or_insert(true);
        *clicked = true;
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_changes_move_node() {
        let node = Node::new(1, Vec2::new(50.0, 50.0));
        let mut changes = Changes::default();

        changes.move_node(&1, &node, Vec2::new(100.0, 100.0));
        let changes_node = changes.nodes.get(&1).unwrap();

        assert_eq!(changes_node.location, Some(Vec2::new(150.0, 150.0)));
    }

    #[test]
    fn test_changes_click_node() {
        let mut changes = Changes::default();

        changes.click_node(&1);
        let changes_node = changes.nodes.get(&1).unwrap();

        assert_eq!(changes_node.clicked, Some(true));
    }

    #[test]
    fn test_changes_select_node() {
        let node = Node::new(1, Vec2::new(50.0, 50.0));
        let mut changes = Changes::default();

        changes.select_node(&1, &node);
        let changes_node = changes.nodes.get(&1).unwrap();

        assert_eq!(changes_node.selected, Some(true));
    }

    #[test]
    fn test_changes_set_dragged_node() {
        let node = Node::new(1, Vec2::new(50.0, 50.0));
        let mut changes = Changes::default();

        changes.set_dragged_node(&1, &node);
        let changes_node = changes.nodes.get(&1).unwrap();

        assert_eq!(changes_node.dragged, Some(true));
    }

    #[test]
    fn test_changes_unset_dragged_node() {
        let node = Node::new(1, Vec2::new(50.0, 50.0));
        let mut changes = Changes::default();

        changes.unset_dragged_node(&1, &node);
        let changes_node = changes.nodes.get(&1).unwrap();

        assert_eq!(changes_node.dragged, Some(false));
    }

    #[test]
    fn test_changes_deselect_node() {
        let node = Node::new(1, Vec2::new(50.0, 50.0));
        let mut changes = Changes::default();

        changes.deselect_node(&1, &node);
        let changes_node = changes.nodes.get(&1).unwrap();

        assert_eq!(changes_node.selected, Some(false));
    }
}
