use std::{
    collections::HashMap,
    f32::{MAX, MIN},
};

use egui::{Color32, Pos2, Rect, Vec2};

use crate::{Changes, ChangesNode};

/// Used to store the state of the graph, i.e. the location of the nodes.
/// It is passed to the GraphView widget and is used to draw the graph.
///
/// Changes to elements should be applied using the `apply_changes` method.
pub struct Elements {
    nodes: HashMap<usize, Node>,
    edges: HashMap<(usize, usize), Vec<Edge>>,
}

impl Elements {
    pub fn new(nodes: HashMap<usize, Node>, edges: HashMap<(usize, usize), Vec<Edge>>) -> Self {
        Self { nodes, edges }
    }

    pub fn get_node(&self, idx: &usize) -> Option<&Node> {
        self.nodes.get(idx)
    }

    pub fn get_nodes(&self) -> &HashMap<usize, Node> {
        &self.nodes
    }

    pub fn get_nodes_mut(&mut self) -> &mut HashMap<usize, Node> {
        &mut self.nodes
    }

    pub fn get_edges(&self) -> &HashMap<(usize, usize), Vec<Edge>> {
        &self.edges
    }

    pub fn get_edges_mut(&mut self) -> &mut HashMap<(usize, usize), Vec<Edge>> {
        &mut self.edges
    }

    /// Applies changes to the current graph.
    ///
    /// The function takes in a reference to a `Changes` struct, which contains
    /// the changes to be made. The changes are applied
    /// sequentially by iterating over the nodes and edges in the `Changes` struct.
    ///
    /// Default changes are applied for the corresponding change, and the user-provided
    /// callback function is called only if there is a change for the corresponding node or edge.
    /// The callback function is applied after the default changes have been applied.
    ///
    /// # Arguments
    ///
    /// * `changes` - A reference to a `Changes` struct containing the changes to be applied;
    /// * `change_node_callback` - A mutable reference to a closure that takes a mutable
    /// reference to `Self` and a reference to a changed node index and `ChangesNode`. This callback is called
    /// after applying changes to each node, if there is a change for the corresponding node.
    pub fn apply_changes<'a>(
        &mut self,
        changes: &'a Changes,
        change_node_callback: &'a mut dyn FnMut(&mut Self, &usize, &ChangesNode),
    ) {
        for (idx, change) in changes.nodes.iter() {
            if let Some(node) = self.get_node_mut(idx) {
                if let Some(location_change) = change.location {
                    node.location = location_change;
                }
                if let Some(radius_change) = change.radius {
                    node.radius = radius_change;
                }
                if let Some(dragged_change) = change.dragged {
                    node.dragged = dragged_change;
                }
                if let Some(selected_change) = change.selected {
                    node.selected = selected_change;
                }

                change_node_callback(self, idx, change);
            }
        }
    }

    /// Returns all directed edges between two nodes as mutable
    pub fn get_edges_between_mut(&mut self, from: &usize, to: &usize) -> Option<&mut Vec<Edge>> {
        self.edges.get_mut(&(*from, *to))
    }

    /// Returns all directed edges between two nodes
    pub fn get_edges_between(&self, from: &usize, to: &usize) -> Option<&Vec<Edge>> {
        self.edges.get(&(*from, *to))
    }

    /// Returns edge at index (from, to, edge_index)
    pub fn get_edge(&self, idx: &(usize, usize, usize)) -> Option<&Edge> {
        self.edges.get(&(idx.0, idx.1))?.get(idx.2)
    }

    /// Deletes node and all edges connected to it
    pub fn delete_node(&mut self, idx: &usize) {
        self.nodes.remove(idx);
        self.edges.retain(|k, _| k.0 != *idx && k.1 != *idx);
    }

    /// Deletes edge
    pub fn delete_edge(&mut self, idx: &(usize, usize, usize)) {
        self.edges.get_mut(&(idx.0, idx.1)).unwrap().remove(idx.2);
    }

    /// Should be used to modify node, mostly when applying changes from the GraphView widget
    pub fn get_node_mut(&mut self, idx: &usize) -> Option<&mut Node> {
        self.nodes.get_mut(idx)
    }

    /// Should be used to modify edge, mostly when applying changes from the GraphView widget
    pub fn get_edge_mut(&mut self, idx: &(usize, usize, usize)) -> Option<&mut Edge> {
        self.edges.get_mut(&(idx.0, idx.1))?.get_mut(idx.2)
    }

    /// Computes the bounds of the graph, i.e. the minimum and maximum x and y coordinates. It also account for the radius of the nodes.
    pub(crate) fn graph_bounds(&self) -> Rect {
        let (mut min_x, mut min_y, mut max_x, mut max_y) = (MAX, MAX, MIN, MIN);

        self.get_nodes().iter().for_each(|(_, n)| {
            let x_minus_rad = n.location.x - n.radius;
            if x_minus_rad < min_x {
                min_x = x_minus_rad;
            };

            let y_minus_rad = n.location.y - n.radius;
            if y_minus_rad < min_y {
                min_y = y_minus_rad;
            };

            let x_plus_rad = n.location.x + n.radius;
            if x_plus_rad > max_x {
                max_x = x_plus_rad;
            };

            let y_plus_rad = n.location.y + n.radius;
            if y_plus_rad > max_y {
                max_y = y_plus_rad;
            };
        });

        Rect::from_min_max(Pos2::new(min_x, min_y), Pos2::new(max_x, max_y))
    }
}

/// Stores properties of a node that can be changed. Used to apply changes to the graph.
#[derive(Clone, Debug)]
pub struct Node {
    pub id: usize,
    pub location: Vec2,

    /// If `color` is None default color is used.
    pub color: Option<Color32>,
    pub radius: f32,

    pub selected: bool,
    pub dragged: bool,
}

impl Node {
    pub fn new(id: usize, location: Vec2) -> Self {
        Self {
            id,
            location,

            color: None,
            radius: 5.,

            selected: false,
            dragged: false,
        }
    }

    pub fn screen_transform(&self, zoom: f32, pan: Vec2) -> Self {
        Self {
            location: self.location * zoom + pan,
            radius: self.radius * zoom,

            id: self.id,
            color: self.color,
            selected: self.selected,
            dragged: self.dragged,
        }
    }
}

/// Stores properties of an edge that can be changed. Used to apply changes to the graph.
#[derive(Clone, Debug)]
pub struct Edge {
    pub start: usize,
    pub end: usize,
    pub list_idx: usize,

    pub width: f32,
    pub tip_size: f32,
    pub tip_angle: f32,
    pub curve_size: f32,

    /// If `color` is None default color is used.
    pub color: Option<Color32>,
    pub selected: bool,
}

impl Edge {
    pub fn new(start: usize, end: usize, list_idx: usize) -> Self {
        Self {
            start,
            end,
            list_idx,

            width: 2.,
            tip_size: 15.,
            tip_angle: std::f32::consts::TAU / 50.,
            curve_size: 20.,

            color: None,
            selected: false,
        }
    }

    pub fn id(&self) -> (usize, usize, usize) {
        (self.start, self.end, self.list_idx)
    }

    pub fn screen_transform(&self, zoom: f32) -> Self {
        Self {
            width: self.width * zoom,
            tip_size: self.tip_size * zoom,
            curve_size: self.curve_size * zoom,

            start: self.start,
            end: self.end,
            list_idx: self.list_idx,
            color: self.color,
            tip_angle: self.tip_angle,
            selected: self.selected,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_sample_elements() -> Elements {
        let mut nodes = HashMap::new();
        nodes.insert(0, Node::new(0, Vec2::new(0.0, 0.0)));
        nodes.insert(1, Node::new(1, Vec2::new(100.0, 100.0)));
        nodes.insert(2, Node::new(2, Vec2::new(-100.0, -100.0)));
        nodes.insert(3, Node::new(3, Vec2::new(50.0, 50.0)));

        let edges = HashMap::new();

        Elements::new(nodes, edges)
    }

    #[test]
    fn test_graph_bounds() {
        let elements = create_sample_elements();
        let bounds = elements.graph_bounds();

        let expected_min_x = -105.0; // x_minus_rad
        let expected_min_y = -105.0; // y_minus_rad
        let expected_max_x = 105.0; // x_plus_rad
        let expected_max_y = 105.0; // y_plus_rad

        assert_eq!(bounds.min.x, expected_min_x);
        assert_eq!(bounds.min.y, expected_min_y);
        assert_eq!(bounds.max.x, expected_max_x);
        assert_eq!(bounds.max.y, expected_max_y);
    }

    #[test]
    fn test_node_screen_transform() {
        let node = Node::new(0, Vec2::new(10.0, 20.0));

        let zoom = 2.0;
        let pan = Vec2::new(5.0, 5.0);
        let transformed_node = node.screen_transform(zoom, pan);

        assert_eq!(transformed_node.location.x, 25.0); // 10 * 2 + 5
        assert_eq!(transformed_node.location.y, 45.0); // 20 * 2 + 5
        assert_eq!(transformed_node.radius, 10.0); // 5 * 2
    }

    #[test]
    fn test_edge_screen_transform() {
        let edge = Edge::new(0, 1, 0);

        let zoom = 3.0;
        let transformed_edge = edge.screen_transform(zoom);

        assert_eq!(transformed_edge.width, 6.0); // 2 * 3
        assert_eq!(transformed_edge.tip_size, 45.0); // 15 * 3
        assert_eq!(transformed_edge.curve_size, 60.0); // 20 * 3
    }

    #[test]
    fn test_apply_changes() {
        let mut elements = create_sample_elements();

        let mut changes = Changes::default();
        changes.move_node(&1, elements.get_node(&1).unwrap(), Vec2::new(100.0, 100.0));
        changes.set_dragged_node(&1, elements.get_node(&1).unwrap());
        changes.select_node(&2, elements.get_node(&2).unwrap());

        let mut change_node_callback =
            |elements: &mut Elements, idx: &usize, _change: &ChangesNode| {
                let node = elements.get_node_mut(idx).unwrap();
                node.color = Some(Color32::RED);
            };

        elements.apply_changes(&changes, &mut change_node_callback);

        let node1 = elements.get_node(&1).unwrap();
        assert_eq!(node1.location, Vec2::new(200.0, 200.0));
        assert!(node1.dragged);
        assert_eq!(node1.color, Some(Color32::RED));

        let node2 = elements.get_node(&2).unwrap();
        assert!(node2.selected);
        assert_eq!(node2.color, Some(Color32::RED));

        let node3 = elements.get_node(&3).unwrap();
        assert_eq!(node3.color, None);
    }
}
