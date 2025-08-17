use std::collections::HashSet;

use egui::Pos2;
use petgraph::{
    csr::IndexType,
    stable_graph::NodeIndex,
    Direction::{Incoming, Outgoing},
    EdgeType,
};
use serde::{Deserialize, Serialize};

use crate::{
    layouts::{Layout, LayoutState},
    DisplayEdge, DisplayNode, Graph,
};

/// Orientation of the hierarchical layout.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum Orientation {
    /// Levels grow downward (classic top-down tree). Rows are vertical steps.
    #[default]
    TopDown,
    /// Levels grow to the right. Rows are horizontal steps.
    LeftRight,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct State {
    /// Run only once unless reset via GraphView::reset_layout or by setting `triggered = false`.
    pub triggered: bool,
    /// Distance between levels (rows). Interpreted as Y step for TopDown and X step for LeftRight.
    pub row_dist: f32,
    /// Distance between siblings/columns. Interpreted as X step for TopDown and Y step for LeftRight.
    pub col_dist: f32,
    /// Center a parent above/beside the span of its children.
    pub center_parent: bool,
    /// Layout orientation.
    pub orientation: Orientation,
}

impl Default for State {
    fn default() -> Self {
        Self {
            triggered: false,
            row_dist: 50.0,
            col_dist: 50.0,
            center_parent: false,
            orientation: Orientation::TopDown,
        }
    }
}

impl LayoutState for State {}

#[derive(Debug, Default)]
pub struct Hierarchical {
    state: State,
}

impl Layout<State> for Hierarchical {
    fn next<N, E, Ty, Ix, Dn, De>(&mut self, g: &mut Graph<N, E, Ty, Ix, Dn, De>, _: &egui::Ui)
    where
        N: Clone,
        E: Clone,
        Ty: EdgeType,
        Ix: IndexType,
        Dn: DisplayNode<N, E, Ty, Ix>,
        De: DisplayEdge<N, E, Ty, Ix, Dn>,
    {
        if self.state.triggered {
            return;
        }

        let mut visited = HashSet::new();

        // Place forests starting from all roots (no incoming edges), packing them left-to-right
        // without overlap by advancing the next starting column by the width of each subtree.
        let mut next_col: usize = 0;
        let roots: Vec<NodeIndex<Ix>> = g.g().externals(Incoming).collect();
        for root in &roots {
            if visited.contains(root) {
                continue;
            }
            let curr_max_col = build_tree(g, &mut visited, root, &self.state, 0, next_col);
            next_col = curr_max_col.saturating_add(1);
        }

        // Fallback: if the graph has cycles or no formal roots, lay out any remaining components.
        let all_nodes: Vec<NodeIndex<Ix>> = g.g().node_indices().collect();
        for n in &all_nodes {
            if visited.contains(n) {
                continue;
            }
            let curr_max_col = build_tree(g, &mut visited, n, &self.state, 0, next_col);
            next_col = curr_max_col.saturating_add(1);
        }

        self.state.triggered = true;
    }

    fn state(&self) -> State {
        self.state.clone()
    }

    fn from_state(state: State) -> impl Layout<State> {
        Hierarchical { state }
    }
}

fn build_tree<N, E, Ty, Ix, Dn, De>(
    g: &mut Graph<N, E, Ty, Ix, Dn, De>,
    visited: &mut HashSet<NodeIndex<Ix>>,
    root_idx: &NodeIndex<Ix>,
    state: &State,
    start_row: usize,
    start_col: usize,
) -> usize
where
    N: Clone,
    E: Clone,
    Ty: EdgeType,
    Ix: IndexType,
    Dn: DisplayNode<N, E, Ty, Ix>,
    De: DisplayEdge<N, E, Ty, Ix, Dn>,
{
    // Mark current node as visited to avoid re-entrancy via back-edges/cycles.
    if !visited.contains(root_idx) {
        visited.insert(*root_idx);
    }

    // Traverse children first to compute the horizontal span of this subtree.
    let mut had_child = false;
    let mut max_col = start_col;
    let mut child_col = start_col;

    let children: Vec<NodeIndex<Ix>> = g.g().neighbors_directed(*root_idx, Outgoing).collect();

    for neighbour_idx in children.iter() {
        if visited.contains(neighbour_idx) {
            continue;
        }
        visited.insert(*neighbour_idx);
        had_child = true;

        let child_max_col = build_tree(g, visited, neighbour_idx, state, start_row + 1, child_col);
        if child_max_col > max_col {
            max_col = child_max_col;
        }
        child_col = child_max_col.saturating_add(1);
    }

    // Column where the current node will be placed.
    let place_col = if state.center_parent && had_child {
        // Center above/beside the span [start_col..=max_col]
        (start_col + max_col) / 2
    } else {
        start_col
    };

    // Compute actual coordinates based on orientation.
    let (x, y) = match state.orientation {
        Orientation::TopDown => (
            (place_col as f32) * state.col_dist,
            (start_row as f32) * state.row_dist,
        ),
        Orientation::LeftRight => (
            (start_row as f32) * state.row_dist,
            (place_col as f32) * state.col_dist,
        ),
    };

    let node = &mut g.g_mut()[*root_idx];
    node.set_location(Pos2::new(x, y));

    max_col
}
