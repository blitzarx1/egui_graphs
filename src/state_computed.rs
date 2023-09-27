use std::collections::HashMap;

use egui::{Rect, Vec2};
use petgraph::{stable_graph::EdgeIndex, stable_graph::NodeIndex, EdgeType};

use crate::{Graph, Node, SettingsStyle};

/// The struct stores selections, dragged node and computed elements states.
#[derive(Debug, Clone)]
pub struct StateComputed {
    pub dragged: Option<NodeIndex>,
    pub selected: Vec<NodeIndex>,
    pub nodes: HashMap<NodeIndex, StateComputedNode>,
    pub edges: HashMap<EdgeIndex, StateComputedEdge>,

    min: Vec2,
    max: Vec2,
    max_rad: f32,
}

impl Default for StateComputed {
    fn default() -> Self {
        Self {
            dragged: None,

            selected: Vec::new(),

            nodes: HashMap::new(),
            edges: HashMap::new(),

            min: Vec2::new(f32::MAX, f32::MAX),
            max: Vec2::new(f32::MIN, f32::MIN),
            max_rad: f32::MIN,
        }
    }
}

impl StateComputed {
    pub fn compute_for_edge(&mut self, idx: EdgeIndex) -> StateComputedEdge {
        *self.edges.entry(idx).or_default()
    }

    pub fn compute_for_node<N: Clone, E: Clone, Ty: EdgeType>(
        &mut self,
        g: &Graph<N, E, Ty>,
        idx: NodeIndex,
    ) -> StateComputedNode {
        let n = g.node(idx).unwrap();
        self.nodes.entry(idx).or_default();

        if n.dragged() {
            self.dragged = Some(idx);
        }

        if n.selected() {
            self.selected.push(idx);
        }

        let comp = self.nodes.get_mut(&idx).unwrap();
        comp.num_connections = g.edges_num(idx);

        *comp
    }

    pub fn comp_iter_bounds<N: Clone>(&mut self, n: &Node<N>, settings: &SettingsStyle) {
        let rad = n.radius() + n.num_connections() as f32 * settings.edge_radius_weight;
        if rad > self.max_rad {
            self.max_rad = rad;
        }

        let loc = n.location();
        if loc.x < self.min.x {
            self.min.x = loc.x;
        };
        if loc.x > self.max.x {
            self.max.x = loc.x;
        };
        if loc.y < self.min.y {
            self.min.y = loc.y;
        };
        if loc.y > self.max.y {
            self.max.y = loc.y;
        };
    }

    pub fn graph_bounds(&self) -> Rect {
        let min = self.min - Vec2::new(self.max_rad, self.max_rad);
        let max = self.max + Vec2::new(self.max_rad, self.max_rad);
        Rect::from_min_max(min.to_pos2(), max.to_pos2())
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub struct StateComputedNode {
    pub selected_child: bool,
    pub selected_parent: bool,
    pub folded_child: bool,
    pub num_folded: usize,
    pub num_connections: usize,
}

#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub struct StateComputedEdge {
    pub selected_child: bool,
    pub selected_parent: bool,
}
