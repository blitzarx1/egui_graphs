use std::{collections::HashMap, rc::Rc, sync::Mutex};

use petgraph::{
    stable_graph::NodeIndex,
    stable_graph::{EdgeIndex, StableGraph},
    EdgeType,
};

use crate::{
    graph_wrapper::GraphWrapper, metadata::Metadata, subgraphs::SubGraphs, Edge, Node,
    SettingsInteraction, SettingsStyle,
};

/// `StateComputed` is a utility struct for managing ephemerial state which is created and destroyed in one frame.
///
/// The struct stores selections, dragged node and computed elements states.
#[derive(Default, Debug, Clone)]
pub struct StateComputed {
    pub dragged: Option<NodeIndex>,
    pub selections: SubGraphs,
    pub foldings: SubGraphs,
    pub nodes: HashMap<NodeIndex, StateComputedNode>,
    pub edges: HashMap<EdgeIndex, StateComputedEdge>,
}

impl StateComputed {
    // pub fn provide_compute_walkers<'a, N: Clone, E: Clone, Ty: EdgeType>(
    //     state: &'a mut StateComputed,
    //     settings_interaction: &'a SettingsInteraction,
    //     settings_style: &'a SettingsStyle,
    // ) -> (
    //     impl FnMut(&'a GraphWrapper<'_, N, E, Ty>, &'a NodeIndex, &'a Node<N>),
    //     impl FnMut(&'a GraphWrapper<'_, N, E, Ty>, &'a EdgeIndex, &'a Edge<E>),
    // ) {
    //     let mut guarded_state = RefCell::new(*state);
    //     (
    //         |g, idx, n| {
    //             guarded_state.get_mut().nodes.entry(*idx).or_default();

    //             // compute radii
    //             let num = g.edges_num(*idx);
    //             let mut radius_addition = settings_style.edge_radius_weight * num as f32;

    //             if n.dragged() {
    //                 guarded_state.get_mut().dragged = Some(*idx);
    //             }

    //             guarded_state.get_mut().compute_selection(
    //                 g,
    //                 *idx,
    //                 n,
    //                 settings_interaction.selection_depth > 0,
    //                 settings_interaction.selection_depth,
    //             );
    //             guarded_state.get_mut().compute_folding(
    //                 g,
    //                 *idx,
    //                 n,
    //                 settings_interaction.folding_depth,
    //             );

    //             radius_addition += guarded_state.get_mut().node_state(&idx).unwrap().num_folded
    //                 as f32
    //                 * settings_style.folded_node_radius_weight;

    //             guarded_state
    //                 .get_mut()
    //                 .nodes
    //                 .get_mut(&idx)
    //                 .unwrap()
    //                 .inc_radius(radius_addition);
    //         },
    //         |_, idx, _| {
    //             guarded_state.get_mut().edges.entry(*idx).or_default();
    //         },
    //     )
    // }

    // TODO: try to use rayon for parallelization of list iterations
    pub fn compute<N: Clone, E: Clone, Ty: EdgeType>(
        g: &GraphWrapper<'_, N, E, Ty>,
        settings_interaction: &SettingsInteraction,
        settings_style: &SettingsStyle,
    ) -> Self {
        let mut state = StateComputed::default();
        g.nodes().for_each(|(idx, n)| {
            state.nodes.entry(idx).or_default();

            // compute radii
            let num = g.edges_num(idx);
            let mut radius_addition = settings_style.edge_radius_weight * num as f32;

            if n.dragged() {
                state.dragged = Some(idx);
            }

            state.compute_selection(
                g,
                idx,
                n,
                settings_interaction.selection_depth > 0,
                settings_interaction.selection_depth,
            );
            state.compute_folding(g, idx, n, settings_interaction.folding_depth);

            radius_addition += state.node_state(&idx).unwrap().num_folded as f32
                * settings_style.folded_node_radius_weight;

            state
                .nodes
                .get_mut(&idx)
                .unwrap()
                .inc_radius(radius_addition);
        });

        g.edges().for_each(|(idx, _)| {
            state.edges.entry(idx).or_default();
        });

        state
    }

    fn compute_selection<N: Clone, E: Clone, Ty: EdgeType>(
        &mut self,
        g: &GraphWrapper<'_, N, E, Ty>,
        root_idx: NodeIndex,
        root: &Node<N>,
        child_mode: bool,
        depth: i32,
    ) {
        if !root.selected() {
            return;
        }

        self.selections.add_subgraph(g, root_idx, depth);

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
            let mut computed = self.edges.entry(*idx).or_default();
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
