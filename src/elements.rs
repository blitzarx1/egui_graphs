use std::{
    collections::HashMap,
    f32::{MAX, MIN},
};

use egui::{Color32, Pos2, Rect, Vec2};
use rand::Rng;

use crate::{Changes, ChangesNode};

/// Used to store the state of the graph, i.e. the location of the nodes.
/// It is passed to the `GraphView` widget and is used to draw the graph.
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

    pub fn node(&self, idx: &usize) -> Option<&Node> {
        self.nodes.get(idx)
    }

    /// Should be used to modify node, mostly when applying changes from the GraphView widget
    pub fn node_mut(&mut self, idx: &usize) -> Option<&mut Node> {
        self.nodes.get_mut(idx)
    }

    pub fn nodes(&self) -> &HashMap<usize, Node> {
        &self.nodes
    }

    pub fn nodes_mut(&mut self) -> &mut HashMap<usize, Node> {
        &mut self.nodes
    }

    pub fn edges(&self) -> &HashMap<(usize, usize), Vec<Edge>> {
        &self.edges
    }

    pub fn edges_mut(&mut self) -> &mut HashMap<(usize, usize), Vec<Edge>> {
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
            if let Some(node) = self.node_mut(idx) {
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

    /// Returns all directed edges between two nodes
    pub fn edges_between(&self, from: &usize, to: &usize) -> Option<&Vec<Edge>> {
        self.edges.get(&(*from, *to))
    }

    /// Returns all directed edges between two nodes as mutable
    pub fn edges_between_mut(&mut self, from: &usize, to: &usize) -> Option<&mut Vec<Edge>> {
        self.edges.get_mut(&(*from, *to))
    }

    /// Returns edge at index (from, to, edge_index)
    pub fn edge(&self, idx: &(usize, usize, usize)) -> Option<&Edge> {
        self.edges.get(&(idx.0, idx.1))?.get(idx.2)
    }

    /// Should be used to modify edge, mostly when applying changes from the GraphView widget
    pub fn edge_mut(&mut self, idx: &(usize, usize, usize)) -> Option<&mut Edge> {
        self.edges.get_mut(&(idx.0, idx.1))?.get_mut(idx.2)
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

    /// Computes the bounds of the graph in real time, i.e. the minimum and maximum x and y coordinates. It also account for the radius of the nodes.
    pub fn rect(&self) -> Rect {
        let (mut min_x, mut min_y, mut max_x, mut max_y) = (MAX, MAX, MIN, MIN);

        self.nodes().iter().for_each(|(_, n)| {
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

    /// Removes node and all edges associated with it.
    pub fn remove_node(&mut self, idx: &usize, neighbors: &[usize]) -> Option<Node> {
        neighbors.iter().for_each(|n| {
            self.remove_edges(idx, n);
            self.remove_edge(n, idx);
        });

        // clear self refenreces
        self.remove_edges(idx, idx);

        self.nodes.remove(idx)
    }

    /// Removes an edge between two nodes. Returns the removed edge if it exists.
    pub fn remove_edge(&mut self, start: &usize, end: &usize) -> Option<Edge> {
        let edges_between = self.edges_between_mut(start, end)?;
        let edge = edges_between.pop()?;
        if edges_between.is_empty() {
            self.edges.remove(&(*start, *end));
        };

        Some(edge)
    }

    /// Adds an edge between two nodes. Returns the added edge if start and end node exist.
    pub fn add_edge(&mut self, start: &usize, end: &usize) -> Option<Edge> {
        self.node(start)?;
        self.node(end)?;

        let edges_between = self.edges_between_mut(start, end);
        if let Some(edges_list) = edges_between {
            let edge_count = edges_list.len();
            let edge = Edge::new(*start, *end, edge_count);
            edges_list.push(edge);

            return Some(edge);
        }

        let edge = Edge::new(*start, *end, 0);
        let edges = self.edges_mut();
        edges.insert((*start, *end), vec![edge]);

        Some(edge)
    }

    /// Removes all edges between two nodes. Returns the removed edges if they exist.
    pub fn remove_edges(
        &mut self,
        start: &usize,
        end: &usize,
    ) -> Option<((usize, usize), Vec<Edge>)> {
        self.edges.remove_entry(&(*start, *end))
    }

    pub fn random_node_idx(&self) -> Option<&usize> {
        if self.nodes.is_empty() {
            return None;
        }
        let mut rng = rand::thread_rng();
        let nodes = self.nodes();
        let random_node_idx = rng.gen_range(0..nodes.len());

        nodes.keys().nth(random_node_idx)
    }

    pub fn random_edge_idx(&self) -> Option<&(usize, usize)> {
        if self.edges.is_empty() {
            return None;
        }
        let mut rng = rand::thread_rng();
        let random_edges_idx = rng.gen_range(0..self.edges.len());

        self.edges.keys().nth(random_edges_idx)
    }
}

/// Stores properties of a node that can be changed. Used to apply changes to the graph.
#[derive(Clone, Debug, Copy, PartialEq)]
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
#[derive(Clone, Debug, Copy, PartialEq)]
pub struct Edge {
    pub id: (usize, usize, usize),
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
            id: (start, end, list_idx),
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

    pub fn screen_transform(&self, zoom: f32) -> Self {
        Self {
            width: self.width * zoom,
            tip_size: self.tip_size * zoom,
            curve_size: self.curve_size * zoom,

            id: self.id,
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
    use std::collections::HashSet;

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
    fn test_rect() {
        let elements = create_sample_elements();
        let bounds = elements.rect();

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
        changes.move_node(&1, elements.node(&1).unwrap(), Vec2::new(100.0, 100.0));
        changes.set_dragged_node(&1, elements.node(&1).unwrap());
        changes.select_node(&2, elements.node(&2).unwrap());

        let mut change_node_callback =
            |elements: &mut Elements, idx: &usize, _change: &ChangesNode| {
                let node = elements.node_mut(idx).unwrap();
                node.color = Some(Color32::RED);
            };

        elements.apply_changes(&changes, &mut change_node_callback);

        let node1 = elements.node(&1).unwrap();
        assert_eq!(node1.location, Vec2::new(200.0, 200.0));
        assert!(node1.dragged);
        assert_eq!(node1.color, Some(Color32::RED));

        let node2 = elements.node(&2).unwrap();
        assert!(node2.selected);
        assert_eq!(node2.color, Some(Color32::RED));

        let node3 = elements.node(&3).unwrap();
        assert_eq!(node3.color, None);
    }

    #[test]
    fn test_add_edge() {
        let mut elements = create_sample_elements();

        // Test adding edge between existing nodes
        let edge = elements.add_edge(&0, &1);
        assert!(edge.is_some());
        let edge = edge.unwrap();
        assert_eq!(edge.start, 0);
        assert_eq!(edge.end, 1);
        assert_eq!(edge.list_idx, 0);

        // Test adding another edge between the same nodes
        let edge2 = elements.add_edge(&0, &1);
        assert!(edge2.is_some());
        let edge2 = edge2.unwrap();
        assert_eq!(edge2.start, 0);
        assert_eq!(edge2.end, 1);
        assert_eq!(edge2.list_idx, 1);

        // Test edge count between nodes
        let edges_between = elements.edges_between(&0, &1);
        assert!(edges_between.is_some());
        let edges_between = edges_between.unwrap();
        assert_eq!(edges_between.len(), 2);

        // Test adding edge between non-existing nodes
        let edge3 = elements.add_edge(&0, &100);
        assert!(edge3.is_none());
    }

    #[test]
    fn test_remove_edges() {
        let mut elements = create_sample_elements();

        elements.add_edge(&0, &1);
        elements.add_edge(&0, &1);
        elements.add_edge(&0, &1);

        assert_eq!(elements.edges_between(&0, &1).unwrap().len(), 3);

        let removed_edges = elements.remove_edges(&0, &1).unwrap();

        assert_eq!(removed_edges.0, (0, 1));
        assert_eq!(removed_edges.1.len(), 3);
        assert!(elements.edges_between(&0, &1).is_none());
    }

    #[test]
    fn test_remove_edge() {
        let mut elements = create_sample_elements();

        elements.add_edge(&1, &2);

        // Test removing edge between existing nodes
        let removed_edge = elements.remove_edge(&1, &2).unwrap();
        assert_eq!(removed_edge.start, 1);
        assert_eq!(removed_edge.end, 2);
        assert_eq!(removed_edge.list_idx, 0);

        // Test removing edge between non-existing nodes
        let non_existing_removed_edge = elements.remove_edge(&1, &10);
        assert_eq!(non_existing_removed_edge, None);
    }

    #[test]
    fn test_remove_node() {
        let mut elements = create_sample_elements();

        // Test removing existing node
        let removed_node = elements.remove_node(&1, &[]).unwrap();
        assert_eq!(removed_node.id, 1);

        // Test removing non-existing node
        let non_existing_removed_node = elements.remove_node(&10, &[]);
        assert_eq!(non_existing_removed_node, None);
    }

    #[test]
    fn test_random_node_idx() {
        let elements = create_sample_elements();

        let mut node_indices = HashSet::new();
        for _ in 0..50 {
            let random_node_idx = elements.random_node_idx();
            node_indices.insert(random_node_idx);
        }

        assert_eq!(node_indices.len(), 4);
    }

    #[test]
    fn test_random_edge_idx() {
        let mut elements = create_sample_elements();

        // Test getting random edge index from non-empty set
        elements.add_edge(&1, &2);
        let random_edge_idx = elements.random_edge_idx().unwrap();
        assert!(elements.edges().contains_key(random_edge_idx));

        // Test getting random edge index from an empty set
        let empty_elements = Elements::new(HashMap::new(), HashMap::new());
        let empty_random_edge_idx = empty_elements.random_edge_idx();
        assert_eq!(empty_random_edge_idx, None);
    }
}
