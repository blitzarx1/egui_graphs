use std::collections::HashMap;

use egui::{Color32, Vec2};

use crate::{Changes, ChangesEdge, ChangesNode};

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

    /// Applies changes to the nodes and edges in the current graph.
    ///
    /// The function takes in a reference to a `Changes` struct, which contains
    /// the changes to be made to the nodes and edges. The changes are applied
    /// sequentially by iterating over the nodes and edges in the `Changes` struct.
    /// Default changes are applied for the corresponding change, and the user-provided
    /// callback functions are called only if there is a change for the corresponding node or edge.
    /// The callback functions are applied after the default changes have been applied.
    ///
    /// # Arguments
    ///
    /// * `changes` - A reference to a `Changes` struct containing the changes to be applied
    /// to the nodes and edges.
    /// * `change_node_callback` - A mutable reference to a closure that takes a mutable
    /// reference to a `Node` and a reference to a `ChangesNode`. This callback is called
    /// after applying changes to each node, if there is a change for the corresponding node.
    /// * `change_edge_callback` - A mutable reference to a closure that takes a mutable
    /// reference to an `Edge` and a reference to a `ChangesEdge`. This callback is called
    /// after applying changes to each edge, if there is a change for the corresponding edge.
    pub fn apply_changes<'a>(
        &mut self,
        changes: &'a Changes,
        change_node_callback: &'a mut dyn FnMut(&mut Node, &ChangesNode),
        change_edge_callback: &'a mut dyn FnMut(&mut Edge, &ChangesEdge),
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

                change_node_callback(node, change);
            }
        }

        for (idx, change) in changes.edges.iter() {
            if let Some(edge) = self.get_edge_mut(idx) {
                if let Some(selected_change) = change.selected {
                    edge.selected = selected_change;
                }

                change_edge_callback(edge, change);
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
}

#[derive(Clone, Debug)]
pub struct Node {
    pub id: usize,
    pub location: Vec2,

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

#[derive(Clone, Debug)]
pub struct Edge {
    pub start: usize,
    pub end: usize,
    pub list_idx: usize,

    pub width: f32,
    pub tip_size: f32,
    pub tip_angle: f32,
    pub curve_size: f32,

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
