use std::collections::HashMap;

use egui::{Pos2, Rect};
use petgraph::{stable_graph::EdgeIndex, stable_graph::NodeIndex, EdgeType};

use crate::{
    metadata::Metadata, settings::SettingsInteraction, subgraphs::SubGraphs, Edge, Graph, Node,
};

/// The struct stores selections, dragged node and computed elements states.
#[derive(Debug, Clone)]
pub struct StateComputed {
    /// Whether we have a node being dragged
    pub has_dragged_node: bool,
    /// Stores the bounds of the graph
    pub graph_bounds: Rect,
    pub dragged: Option<NodeIndex>,
    pub selections: SubGraphs,
    pub foldings: SubGraphs,
    pub nodes: HashMap<NodeIndex, StateComputedNode>,
    pub edges: HashMap<EdgeIndex, StateComputedEdge>,

    min_x: f32,
    min_y: f32,
    max_x: f32,
    max_y: f32,
}

impl Default for StateComputed {
    fn default() -> Self {
        Self {
            has_dragged_node: false,
            graph_bounds: Rect::from_two_pos(egui::Pos2::default(), egui::Pos2::default()),
            dragged: None,
            selections: SubGraphs::default(),
            foldings: SubGraphs::default(),
            nodes: HashMap::new(),
            edges: HashMap::new(),

            min_x: f32::MAX,
            min_y: f32::MAX,
            max_x: f32::MIN,
            max_y: f32::MIN,
        }
    }
}

impl StateComputed {
    pub fn compute_for_edge<E: Clone>(&mut self, idx: EdgeIndex, e: &Edge<E>, m: &Metadata) {
        let comp = self.edges.entry(idx).or_insert(StateComputedEdge::new(e));
        comp.apply_screen_transform(m);
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

        let (x, y) = (n.location().x, n.location().y);
        if x < self.min_x {
            self.min_x = x;
        };
        if y < self.min_y {
            self.min_y = y;
        };
        if x > self.max_x {
            self.max_x = x;
        };
        if y > self.max_y {
            self.max_y = y;
        };

        self.compute_selection(g, idx, n, settings_interaction);
        self.compute_folding(g, idx, n, settings_interaction);

        let comp = self.nodes.get_mut(&idx).unwrap();
        comp.num_connections = g.edges_num(idx);

        *comp
    }

    pub fn compute_graph_bounds(&mut self) {
        self.graph_bounds = Rect::from_min_max(
            Pos2::new(self.min_x, self.min_y),
            Pos2::new(self.max_x, self.max_y),
        );
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
            let computed = self
                .edges
                .entry(*idx)
                .or_insert(StateComputedEdge::new(g.edge(*idx).unwrap()));
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

impl StateComputedNode {
    /// Indicates if node is visible and should be drawn
    pub fn visible(&self) -> bool {
        !self.subfolded()
    }

    pub fn subfolded(&self) -> bool {
        self.folded_child
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StateComputedEdge {
    pub selected_child: bool,
    pub selected_parent: bool,
    pub width: f32,
    pub tip_size: f32,
    pub curve_size: f32,
}

impl StateComputedEdge {
    pub fn new<E: Clone>(e: &Edge<E>) -> Self {
        Self {
            width: e.width(),
            tip_size: e.tip_size(),
            curve_size: e.curve_size(),

            selected_child: Default::default(),
            selected_parent: Default::default(),
        }
    }

    pub fn subselected(&self) -> bool {
        self.selected_child || self.selected_parent
    }

    pub fn apply_screen_transform(&mut self, m: &Metadata) {
        self.width *= m.zoom;
        self.tip_size *= m.zoom;
        self.curve_size *= m.zoom;
    }
}
