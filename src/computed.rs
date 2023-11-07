use egui::{Rect, Vec2};
use petgraph::graph::EdgeIndex;
use petgraph::{stable_graph::NodeIndex, EdgeType};

use crate::{Graph, Node, SettingsStyle};

/// The struct stores selections, dragged node and computed elements states.
#[derive(Debug, Clone)]
pub struct ComputedState {
    pub dragged: Option<NodeIndex>,
    pub selected_nodes: Vec<NodeIndex>,
    pub selected_edges: Vec<EdgeIndex>,

    min: Vec2,
    max: Vec2,
    max_rad: f32,
}

impl Default for ComputedState {
    fn default() -> Self {
        Self {
            dragged: None,

            selected_nodes: Vec::new(),
            selected_edges: Vec::new(),

            min: Vec2::new(f32::MAX, f32::MAX),
            max: Vec2::new(f32::MIN, f32::MIN),
            max_rad: f32::MIN,
        }
    }
}

impl ComputedState {
    pub fn compute_for_node<N: Clone, E: Clone, Ty: EdgeType>(
        &mut self,
        g: &Graph<N, E, Ty>,
        idx: NodeIndex,
    ) -> ComputedNode {
        let n = g.node(idx).unwrap();

        if n.dragged() {
            self.dragged = Some(idx);
        }
        if n.selected() {
            self.selected_nodes.push(idx);
        }

        ComputedNode {
            num_connections: g.edges_num(idx),
        }
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
pub struct ComputedNode {
    pub num_connections: usize,
}
