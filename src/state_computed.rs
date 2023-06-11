use std::{
    collections::HashMap,
    f32::{MAX, MIN},
};

use egui::{Pos2, Rect};
use petgraph::{stable_graph::EdgeIndex, stable_graph::NodeIndex, visit::EdgeRef, EdgeType};

use crate::{
    graph_wrapper::GraphWrapper, metadata::Metadata, subgraphs::SubGraphs, Node,
    SettingsInteraction, SettingsStyle,
};

/// `StateComputed` is a utility struct for managing ephemerial state which is created and destroyed in one frame.
///
/// The struct stores selections, dragged node and computed elements states.
#[derive(Default, Debug, Clone)]
pub struct StateComputed {
    pub dragged: Option<NodeIndex>,
    pub selections: Option<SubGraphs>,
    pub foldings: Option<SubGraphs>,
    pub nodes: HashMap<NodeIndex, StateComputedNode>,
    pub edges: HashMap<EdgeIndex, StateComputedEdge>,
}

impl StateComputed {
    // TODO: try to use rayon for parallelization of list iterations
    pub fn compute<N: Clone, E: Clone, Ty: EdgeType>(
        g: &GraphWrapper<'_, N, E, Ty>,
        settings_interaction: &SettingsInteraction,
        settings_style: &SettingsStyle,
        meta: &mut Metadata,
    ) -> Self {
        let nodes_computed = g.nodes().map(|(idx, _)| {
            let node_state = StateComputedNode::default();
            (idx, node_state)
        });

        let edges_computed = g.edges().map(|(idx, _)| {
            let edge_state = StateComputedEdge::default();
            (idx, edge_state)
        });

        let mut state = StateComputed {
            nodes: nodes_computed.collect(),
            edges: edges_computed.collect(),
            ..Default::default()
        };

        // compute radii and selections
        let mut selections = SubGraphs::default();
        let mut foldings = SubGraphs::default();
        let mut new_dragged = None;
        let (mut min_x, mut min_y, mut max_x, mut max_y) = (MAX, MAX, MIN, MIN);
        g.nodes().for_each(|(root_idx, root_n)| {
            // compute radii
            let num = g.edges_num(root_idx);
            let mut radius_addition = settings_style.edge_radius_weight * num as f32;

            if root_n.dragged() {
                new_dragged = Some(root_idx);
            }

            state.compute_selection(
                g,
                &mut selections,
                root_idx,
                root_n,
                settings_interaction.selection_depth > 0,
                settings_interaction.selection_depth,
            );
            state.compute_folding(
                g,
                &mut foldings,
                root_idx,
                root_n,
                settings_interaction.folding_depth,
            );

            radius_addition += state.node_state(&root_idx).unwrap().num_folded as f32
                * settings_style.folded_node_radius_weight;

            state
                .node_state_mut(&root_idx)
                .unwrap()
                .inc_radius(radius_addition);

            let comp_node = state.node_state(&root_idx).unwrap();

            let x_minus_rad = root_n.location().x - comp_node.radius(meta);
            if x_minus_rad < min_x {
                min_x = x_minus_rad;
            };

            let y_minus_rad = root_n.location().y - comp_node.radius(meta);
            if y_minus_rad < min_y {
                min_y = y_minus_rad;
            };

            let x_plus_rad = root_n.location().x + comp_node.radius(meta);
            if x_plus_rad > max_x {
                max_x = x_plus_rad;
            };

            let y_plus_rad = root_n.location().y + comp_node.radius(meta);
            if y_plus_rad > max_y {
                max_y = y_plus_rad;
            };
        });

        state.dragged = new_dragged;
        state.selections = Some(selections);
        state.foldings = Some(foldings);

        meta.graph_bounds = Rect::from_min_max(Pos2::new(min_x, min_y), Pos2::new(max_x, max_y));

        state
    }

    fn compute_selection<N: Clone, E: Clone, Ty: EdgeType>(
        &mut self,
        g: &GraphWrapper<'_, N, E, Ty>,
        selections: &mut SubGraphs,
        root_idx: NodeIndex,
        root: &Node<N>,
        child_mode: bool,
        depth: i32,
    ) {
        if !root.selected() {
            return;
        }

        selections.add_subgraph(g, root_idx, depth);

        let elements = selections.elements_by_root(root_idx);
        if elements.is_none() {
            return;
        }

        let (nodes, edges) = elements.unwrap();

        nodes.iter().for_each(|idx| {
            if *idx == root_idx {
                return;
            }

            let computed = self.node_state_mut(idx).unwrap();
            if child_mode {
                computed.selected_child = true;
                return;
            }
            computed.selected_parent = true;
        });

        edges.iter().for_each(|idx| {
            let mut computed = self.edge_state_mut(idx).unwrap();
            if child_mode {
                computed.selected_child = true;
                return;
            }
            computed.selected_parent = true;
        });
    }

    fn compute_folding<N: Clone, E: Clone, Ty: EdgeType>(
        &mut self,
        g: &GraphWrapper<'_, N, E, Ty>,
        foldings: &mut SubGraphs,
        root_idx: NodeIndex,
        root: &Node<N>,
        depth: usize,
    ) {
        if !root.folded() {
            return;
        }

        let depth_normalized = match depth {
            usize::MAX => i32::MAX,
            _ => depth as i32,
        };

        foldings.add_subgraph(g, root_idx, depth_normalized);

        let elements = foldings.elements_by_root(root_idx);
        if elements.is_none() {
            return;
        }

        let (nodes, _) = elements.unwrap();
        self.node_state_mut(&root_idx).unwrap().num_folded = nodes.len() - 1; // dont't count root node

        nodes.iter().for_each(|idx| {
            if *idx == root_idx {
                return;
            }

            let computed = self.node_state_mut(idx).unwrap();
            computed.folded_child = true;
        });
    }

    pub fn node_state(&self, idx: &NodeIndex) -> Option<&StateComputedNode> {
        self.nodes.get(idx)
    }

    pub fn node_state_mut(&mut self, idx: &NodeIndex) -> Option<&mut StateComputedNode> {
        self.nodes.get_mut(idx)
    }

    pub fn edge_state(&self, idx: &EdgeIndex) -> Option<&StateComputedEdge> {
        self.edges.get(idx)
    }

    pub fn edge_state_mut(&mut self, idx: &EdgeIndex) -> Option<&mut StateComputedEdge> {
        self.edges.get_mut(idx)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StateComputedNode {
    pub selected_child: bool,
    pub selected_parent: bool,
    pub folded_child: bool,
    pub num_folded: usize,
    radius: f32,
}

impl Default for StateComputedNode {
    fn default() -> Self {
        Self {
            selected_child: Default::default(),
            selected_parent: Default::default(),
            folded_child: Default::default(),
            num_folded: Default::default(),
            radius: 5.,
        }
    }
}

impl StateComputedNode {
    pub fn subselected(&self) -> bool {
        self.selected_child || self.selected_parent
    }

    pub fn subfolded(&self) -> bool {
        self.folded_child
    }

    pub fn radius(&self, meta: &Metadata) -> f32 {
        self.radius * meta.zoom
    }

    pub fn inc_radius(&mut self, inc: f32) {
        self.radius += inc;
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub struct StateComputedEdge {
    pub selected_child: bool,
    pub selected_parent: bool,
}

impl StateComputedEdge {
    pub fn subselected(&self) -> bool {
        self.selected_child || self.selected_parent
    }
}
