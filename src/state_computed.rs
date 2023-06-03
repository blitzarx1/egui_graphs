use std::collections::HashMap;

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
    // TODO: compute foldings
    pub fn compute<'a, N: Clone, E: Clone, Ty: EdgeType>(
        g: &GraphWrapper<'a, N, E, Ty>,
        settings_interaction: &SettingsInteraction,
        settings_style: &SettingsStyle,
    ) -> Self {
        let nodes_computed = g.nodes().map(|(idx, _)| {
            let node_state = StateComputedNode::default();
            (idx, node_state)
        });

        let edges_computed = g.edges().map(|e| {
            let edge_state = StateComputedEdge::default();
            (e.id(), edge_state)
        });

        let mut state = StateComputed {
            nodes: nodes_computed.collect(),
            edges: edges_computed.collect(),
            ..Default::default()
        };

        // compute radii and selections
        let mut selections = SubGraphs::default();
        let mut foldings = SubGraphs::default();
        g.nodes().for_each(|(root_idx, root_n)| {
            // TODO: remove raddii comp from subgraphs iter
            // compute radii
            let num = g.edges_num(root_idx);
            state
                .node_state_mut(&root_idx)
                .unwrap()
                .inc_radius(settings_style.edge_radius_weight * num as f32);

            state.compute_selection(
                g,
                &mut selections,
                root_idx,
                root_n,
                settings_interaction.selection_depth > 0,
                settings_interaction.selection_depth,
            )
        });

        state.selections = Some(selections);
        state.foldings = Some(foldings);

        state
    }

    fn compute_selection<'a, N: Clone, E: Clone, Ty: EdgeType>(
        &mut self,
        g: &GraphWrapper<'a, N, E, Ty>,
        selections: &mut SubGraphs,
        root_idx: NodeIndex,
        root: &Node<N>,
        child_mode: bool,
        depth: i32,
    ) {
        if !root.selected {
            return;
        }

        selections.add_subgraph(&g, root_idx, depth);

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
    radius: f32,
}

impl Default for StateComputedNode {
    fn default() -> Self {
        Self {
            selected_child: Default::default(),
            selected_parent: Default::default(),
            radius: 5.,
        }
    }
}

impl StateComputedNode {
    pub fn subselected(&self) -> bool {
        self.selected_child || self.selected_parent
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
