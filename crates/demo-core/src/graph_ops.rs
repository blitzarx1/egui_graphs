use crate::{MAX_EDGE_COUNT, MAX_NODE_COUNT};
use egui::Pos2;
use egui_graphs::Graph;
use petgraph::stable_graph::{DefaultIx, EdgeIndex, NodeIndex};
use petgraph::Directed;
use rand::Rng;

pub struct GraphActions<'a> {
    pub g: &'a mut Graph<(), (), Directed, DefaultIx>,
}

impl GraphActions<'_> {
    pub fn add_nodes(&mut self, n: u32) {
        for _ in 0..n {
            self.add_random_node();
        }
    }
    pub fn remove_nodes(&mut self, n: u32) {
        for _ in 0..n {
            self.remove_random_node();
        }
    }
    pub fn swap_nodes(&mut self, n: u32) {
        for _ in 0..n {
            self.remove_random_node();
            self.add_random_node();
        }
    }
    pub fn add_edges(&mut self, n: u32) {
        for _ in 0..n {
            self.add_random_edge();
        }
    }
    pub fn remove_edges(&mut self, n: u32) {
        for _ in 0..n {
            self.remove_random_edge();
        }
    }
    pub fn swap_edges(&mut self, n: u32) {
        for _ in 0..n {
            self.remove_random_edge();
            self.add_random_edge();
        }
    }

    pub fn add_random_node(&mut self) {
        if self.g.node_count() >= MAX_NODE_COUNT {
            return;
        }
        let base = if let Some(r) = self.random_node_idx() {
            self.g.node(r).unwrap().location()
        } else {
            Pos2::new(0.0, 0.0)
        };
        let mut rng = rand::rng();
        let loc = Pos2::new(
            base.x + rng.random_range(-150.0..150.0),
            base.y + rng.random_range(-150.0..150.0),
        );
        self.g.add_node_with_location((), loc);
    }
    pub fn remove_random_node(&mut self) {
        if let Some(i) = self.random_node_idx() {
            self.g.remove_node(i);
        }
    }
    pub fn add_random_edge(&mut self) {
        if let (Some(a), Some(b)) = (self.random_node_idx(), self.random_node_idx()) {
            self.add_edge(a, b);
        }
    }
    pub fn add_edge(&mut self, a: NodeIndex, b: NodeIndex) {
        if self.g.edge_count() < MAX_EDGE_COUNT {
            self.g.add_edge(a, b, ());
        }
    }
    pub fn remove_random_edge(&mut self) {
        if let Some(eidx) = self.random_edge_idx() {
            if let Some((a, b)) = self.g.edge_endpoints(eidx) {
                self.remove_edge(a, b);
            }
        }
    }
    pub fn remove_edge(&mut self, a: NodeIndex, b: NodeIndex) {
        let edge_id_opt = self.g.edges_connecting(a, b).next().map(|(eid, _)| eid);
        if let Some(edge_id) = edge_id_opt {
            self.g.remove_edge(edge_id);
        }
    }

    fn random_node_idx(&self) -> Option<NodeIndex> {
        let cnt = self.g.node_count();
        if cnt == 0 {
            return None;
        }
        let idx = rand::rng().random_range(0..cnt);
        self.g.g().node_indices().nth(idx)
    }
    fn random_edge_idx(&self) -> Option<EdgeIndex> {
        let cnt = self.g.edge_count();
        if cnt == 0 {
            return None;
        }
        let idx = rand::rng().random_range(0..cnt);
        self.g.g().edge_indices().nth(idx)
    }
}
