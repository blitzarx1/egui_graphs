use crate::{DemoGraph, MAX_EDGE_COUNT, MAX_NODE_COUNT};
use egui::Pos2;
use petgraph::stable_graph::{EdgeIndex, NodeIndex};
use rand::Rng;

pub struct GraphActions<'a> {
    pub g: &'a mut DemoGraph,
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
        let node_cnt = match self.g {
            DemoGraph::Directed(ref g) => g.node_count(),
            DemoGraph::Undirected(ref g) => g.node_count(),
        };
        if node_cnt >= MAX_NODE_COUNT {
            return;
        }
        let base = match self.random_node_idx() {
            Some(r) => match self.g {
                DemoGraph::Directed(ref g) => g.node(r).unwrap().location(),
                DemoGraph::Undirected(ref g) => g.node(r).unwrap().location(),
            },
            None => Pos2::new(0.0, 0.0),
        };
        let mut rng = rand::rng();
        let loc = Pos2::new(
            base.x + rng.random_range(-150.0..150.0),
            base.y + rng.random_range(-150.0..150.0),
        );
        match self.g {
            DemoGraph::Directed(ref mut g) => {
                g.add_node_with_location((), loc);
            }
            DemoGraph::Undirected(ref mut g) => {
                g.add_node_with_location((), loc);
            }
        }
    }
    pub fn remove_random_node(&mut self) {
        if let Some(i) = self.random_node_idx() {
            match self.g {
                DemoGraph::Directed(ref mut g) => {
                    g.remove_node(i);
                }
                DemoGraph::Undirected(ref mut g) => {
                    g.remove_node(i);
                }
            }
        }
    }
    pub fn add_random_edge(&mut self) {
        if let (Some(a), Some(b)) = (self.random_node_idx(), self.random_node_idx()) {
            self.add_edge(a, b);
        }
    }
    pub fn add_edge(&mut self, a: NodeIndex, b: NodeIndex) {
        let edge_cnt = match self.g {
            DemoGraph::Directed(ref g) => g.edge_count(),
            DemoGraph::Undirected(ref g) => g.edge_count(),
        };
        if edge_cnt >= MAX_EDGE_COUNT {
            return;
        }
        match self.g {
            DemoGraph::Directed(ref mut g) => {
                g.add_edge(a, b, ());
            }
            DemoGraph::Undirected(ref mut g) => {
                g.add_edge(a, b, ());
            }
        }
    }
    pub fn remove_random_edge(&mut self) {
        if let Some(eidx) = self.random_edge_idx() {
            let endpoints = match self.g {
                DemoGraph::Directed(ref g) => g.edge_endpoints(eidx),
                DemoGraph::Undirected(ref g) => g.edge_endpoints(eidx),
            };
            if let Some((a, b)) = endpoints {
                self.remove_edge(a, b);
            }
        }
    }
    pub fn remove_edge(&mut self, a: NodeIndex, b: NodeIndex) {
        let edge_id_opt = match self.g {
            DemoGraph::Directed(ref g) => g.edges_connecting(a, b).next().map(|(eid, _)| eid),
            DemoGraph::Undirected(ref g) => g.edges_connecting(a, b).next().map(|(eid, _)| eid),
        };
        if let Some(edge_id) = edge_id_opt {
            match self.g {
                DemoGraph::Directed(ref mut g) => {
                    g.remove_edge(edge_id);
                }
                DemoGraph::Undirected(ref mut g) => {
                    g.remove_edge(edge_id);
                }
            }
        }
    }
    pub fn remove_node_by_idx(&mut self, idx: NodeIndex) {
        match self.g {
            DemoGraph::Directed(ref mut g) => {
                g.remove_node(idx);
            }
            DemoGraph::Undirected(ref mut g) => {
                g.remove_node(idx);
            }
        }
    }

    fn random_node_idx(&self) -> Option<NodeIndex> {
        let cnt = match self.g {
            DemoGraph::Directed(ref g) => g.node_count(),
            DemoGraph::Undirected(ref g) => g.node_count(),
        };
        if cnt == 0 {
            return None;
        }
        let idx = rand::rng().random_range(0..cnt);
        match self.g {
            DemoGraph::Directed(ref g) => g.g().node_indices().nth(idx),
            DemoGraph::Undirected(ref g) => g.g().node_indices().nth(idx),
        }
    }
    fn random_edge_idx(&self) -> Option<EdgeIndex> {
        let cnt = match self.g {
            DemoGraph::Directed(ref g) => g.edge_count(),
            DemoGraph::Undirected(ref g) => g.edge_count(),
        };
        if cnt == 0 {
            return None;
        }
        let idx = rand::rng().random_range(0..cnt);
        match self.g {
            DemoGraph::Directed(ref g) => g.g().edge_indices().nth(idx),
            DemoGraph::Undirected(ref g) => g.g().edge_indices().nth(idx),
        }
    }
}
