use std::collections::HashMap;

use egui::{Rect, Vec2};
use petgraph::{stable_graph::EdgeIndex, stable_graph::NodeIndex, EdgeType};

use crate::{settings::SettingsInteraction, subgraphs::SubGraphs, Graph, Node, SettingsStyle};

/// The struct stores selections, dragged node and computed elements states.
#[derive(Debug, Clone)]
pub struct StateComputed {
    pub dragged: Option<NodeIndex>,
    pub selections: SubGraphs,
    pub foldings: SubGraphs,
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
            selections: SubGraphs::default(),
            foldings: SubGraphs::default(),
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
        settings_interaction: &SettingsInteraction,
    ) -> StateComputedNode {
        let n = g.node(idx).unwrap();
        self.nodes.entry(idx).or_default();

        if n.dragged() {
            self.dragged = Some(idx);
        }

        self.compute_selection(g, idx, n, settings_interaction);
        self.compute_folding(g, idx, n, settings_interaction);

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

    fn compute_selection<N: Clone, E: Clone, Ty: EdgeType>(
        &mut self,
        g: &Graph<N, E, Ty>,
        root_idx: NodeIndex,
        root: &Node<N>,
        settings_interaction: &SettingsInteraction,
    ) {
        if !root.selected() {
            return;
        }

        let child_mode = settings_interaction.selection_depth > 0;
        self.selections
            .add_subgraph(g, root_idx, settings_interaction.selection_depth);

        let elements = self.selections.elements_by_root(root_idx);
        if elements.is_none() {
            return;
        }

        let (nodes, edges) = elements.unwrap();

        nodes.iter().for_each(|idx| {
            if *idx == root_idx {
                return;
            }

            let computed = self.nodes.entry(*idx).or_default();
            if child_mode {
                computed.selected_child = true;
                return;
            }
            computed.selected_parent = true;
        });

        edges.iter().for_each(|idx| {
            let computed = self.edges.entry(*idx).or_default();
            if child_mode {
                computed.selected_child = true;
                return;
            }
            computed.selected_parent = true;
        });
    }

    fn compute_folding<N: Clone, E: Clone, Ty: EdgeType>(
        &mut self,
        g: &Graph<N, E, Ty>,
        root_idx: NodeIndex,
        root: &Node<N>,
        settings_interaction: &SettingsInteraction,
    ) {
        if !root.folded() {
            return;
        }

        let depth_normalized = match settings_interaction.folding_depth {
            usize::MAX => i32::MAX,
            _ => settings_interaction.folding_depth as i32,
        };

        self.foldings.add_subgraph(g, root_idx, depth_normalized);

        let elements = self.foldings.elements_by_root(root_idx);
        if elements.is_none() {
            return;
        }

        let (nodes, _) = elements.unwrap();
        self.nodes.entry(root_idx).or_default().num_folded = nodes.len() - 1; // dont't count root node

        nodes.iter().for_each(|idx| {
            if *idx == root_idx {
                return;
            }

            self.nodes.entry(*idx).or_default().folded_child = true;
        });
    }

    pub fn node_state(&self, idx: &NodeIndex) -> Option<&StateComputedNode> {
        self.nodes.get(idx)
    }

    pub fn edge_state(&self, idx: &EdgeIndex) -> Option<&StateComputedEdge> {
        self.edges.get(idx)
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
