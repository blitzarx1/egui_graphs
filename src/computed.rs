use egui::{Rect, Vec2};
use petgraph::graph::{EdgeIndex, IndexType};
use petgraph::{stable_graph::NodeIndex, EdgeType};

use crate::{Graph, Node};

/// The struct stores selections, dragged node and computed elements states.
#[derive(Debug, Clone)]
pub struct ComputedState<Ix: IndexType> {
    pub dragged: Option<NodeIndex<Ix>>,
    pub selected_nodes: Vec<NodeIndex<Ix>>,
    pub selected_edges: Vec<EdgeIndex<Ix>>,

    min: Vec2,
    max: Vec2,
}

impl<Ix> Default for ComputedState<Ix>
where
    Ix: IndexType,
{
    fn default() -> Self {
        Self {
            dragged: None,

            selected_nodes: Vec::new(),
            selected_edges: Vec::new(),

            min: Vec2::new(f32::MAX, f32::MAX),
            max: Vec2::new(f32::MIN, f32::MIN),
        }
    }
}

impl<Ix> ComputedState<Ix>
where
    Ix: IndexType,
{
    pub fn compute_for_node<N: Clone, E: Clone, Ty: EdgeType>(
        &mut self,
        g: &Graph<N, E, Ty, Ix>,
        idx: NodeIndex<Ix>,
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

    pub fn comp_iter_bounds<N: Clone>(&mut self, n: &Node<N, Ix>) {
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
        Rect::from_min_max(self.min.to_pos2(), self.max.to_pos2())
    }
}

#[cfg_attr(feature="serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub struct ComputedNode {
    pub num_connections: usize,
}
