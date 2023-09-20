use std::collections::HashMap;

use egui::{Pos2, Rect, Vec2};
use petgraph::{stable_graph::EdgeIndex, stable_graph::NodeIndex, EdgeType};

use crate::{
    metadata::Metadata,
    settings::{SettingsInteraction, SettingsStyle},
    subgraphs::SubGraphs,
    Edge, Graph, Node,
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
        meta: &Metadata,
        settings_interaction: &SettingsInteraction,
        settings_style: &SettingsStyle,
    ) -> StateComputedNode {
        // compute radii
        let num = g.edges_num(idx);
        let n = g.node(idx).unwrap();
        self.nodes
            .entry(idx)
            .or_insert(StateComputedNode::new(n, settings_style));

        let mut radius_addition = settings_style.edge_radius_weight * num as f32;

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

        self.compute_selection(g, idx, n, settings_interaction, settings_style);
        self.compute_folding(g, idx, n, settings_interaction, settings_style);

        radius_addition +=
            self.node_state(&idx).unwrap().num_folded as f32 * settings_style.folded_radius_weight;

        let comp = self.nodes.get_mut(&idx).unwrap();
        comp.inc_radius(radius_addition);
        comp.apply_screen_transform(meta);
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
        settings_style: &SettingsStyle,
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

            let computed = self.nodes.entry(*idx).or_insert(StateComputedNode::new(
                g.node(*idx).unwrap(),
                settings_style,
            ));
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
        settings_style: &SettingsStyle,
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
        self.nodes
            .entry(root_idx)
            .or_insert(StateComputedNode::new(root, settings_style))
            .num_folded = nodes.len() - 1; // dont't count root node

        nodes.iter().for_each(|idx| {
            if *idx == root_idx {
                return;
            }

            self.nodes
                .entry(*idx)
                .or_insert(StateComputedNode::new(
                    g.node(*idx).unwrap(),
                    settings_style,
                ))
                .folded_child = true;
        });
    }

    pub fn node_state(&self, idx: &NodeIndex) -> Option<&StateComputedNode> {
        self.nodes.get(idx)
    }

    pub fn edge_state(&self, idx: &EdgeIndex) -> Option<&StateComputedEdge> {
        self.edges.get(idx)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StateComputedNode {
    pub selected_child: bool,
    pub selected_parent: bool,
    pub folded_child: bool,
    pub num_folded: usize,
    pub location: Vec2,
    pub radius: f32,
}

impl StateComputedNode {
    pub fn new<N: Clone>(n: &Node<N>, settings_style: &SettingsStyle) -> Self {
        Self {
            location: n.location(),
            radius: settings_style.node_radius,

            selected_child: Default::default(),
            selected_parent: Default::default(),
            folded_child: Default::default(),
            num_folded: Default::default(),
        }
    }

    /// Indicates if node is visible and should be drawn
    pub fn visible(&self) -> bool {
        !self.subfolded()
    }

    pub fn subfolded(&self) -> bool {
        self.folded_child
    }

    pub fn inc_radius(&mut self, inc: f32) {
        self.radius += inc;
    }

    pub fn apply_screen_transform(&mut self, m: &Metadata) {
        self.location = self.location * m.zoom + m.pan;
        self.radius *= m.zoom;
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
