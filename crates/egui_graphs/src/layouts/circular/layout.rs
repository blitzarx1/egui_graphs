use egui;
use serde::{Deserialize, Serialize};

use crate::graph::Graph;
use crate::layouts::{Layout, LayoutState};
use crate::{DisplayEdge, DisplayNode};
use petgraph::graph::IndexType;
use petgraph::EdgeType;

/// State for the circular layout algorithm
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct State {
    applied: bool,
}

impl LayoutState for State {}

/// Sort order for circular layout nodes
#[derive(Debug, Clone)]
pub enum SortOrder {
    /// Alphabetical by label (ascending)
    Alphabetical,
    /// Reverse alphabetical by label (descending)
    ReverseAlphabetical,
    /// No sorting - preserve insertion order
    None,
}

impl Default for SortOrder {
    fn default() -> Self {
        SortOrder::Alphabetical
    }
}

/// Configuration for spacing/radius of the circular layout
#[derive(Debug, Clone)]
pub struct SpacingConfig {
    /// Base radius when there are few nodes
    pub base_radius: f32,
    /// Additional radius per node (for auto-scaling)
    pub radius_per_node: f32,
    /// If set, overrides the auto-calculated radius
    pub fixed_radius: Option<f32>,
}

impl Default for SpacingConfig {
    fn default() -> Self {
        Self {
            base_radius: 50.0,
            radius_per_node: 5.0,
            fixed_radius: None,
        }
    }
}

impl SpacingConfig {
    /// Set the base radius for the circle
    pub fn with_base_radius(mut self, base: f32) -> Self {
        self.base_radius = base;
        self
    }

    /// Set the additional radius per node for auto-scaling
    pub fn with_radius_per_node(mut self, per_node: f32) -> Self {
        self.radius_per_node = per_node;
        self
    }

    /// Set a fixed radius, overriding auto-scaling
    pub fn with_fixed_radius(mut self, radius: f32) -> Self {
        self.fixed_radius = Some(radius);
        self
    }
}

/// Circular layout arranges nodes in a circle.
///
/// Nodes are positioned evenly around a circle with configurable:
///
/// - Sort order (alphabetical, reverse, or insertion order)
/// - Spacing (auto-scaling or fixed radius)
///
/// The layout applies once and preserves the circular arrangement.
#[derive(Debug, Clone)]
pub struct Circular {
    state: State,
    sort_order: SortOrder,
    spacing: SpacingConfig,
}

impl Default for Circular {
    fn default() -> Self {
        Self {
            state: State::default(),
            sort_order: SortOrder::default(),
            spacing: SpacingConfig::default(),
        }
    }
}

impl Circular {
    /// Create a new circular layout with default configuration
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the sort order for nodes around the circle
    pub fn with_sort_order(mut self, sort_order: SortOrder) -> Self {
        self.sort_order = sort_order;
        self
    }

    /// Disable sorting, preserving insertion order
    pub fn without_sorting(mut self) -> Self {
        self.sort_order = SortOrder::None;
        self
    }

    /// Set custom spacing configuration
    pub fn with_spacing(mut self, spacing: SpacingConfig) -> Self {
        self.spacing = spacing;
        self
    }
}

impl Layout<State> for Circular {
    fn from_state(state: State) -> impl Layout<State> {
        Self {
            state,
            sort_order: SortOrder::default(),
            spacing: SpacingConfig::default(),
        }
    }

    fn next<N, E, Ty, Ix, Dn, De>(&mut self, g: &mut Graph<N, E, Ty, Ix, Dn, De>, ui: &egui::Ui)
    where
        N: Clone,
        E: Clone,
        Ty: EdgeType,
        Ix: IndexType,
        Dn: DisplayNode<N, E, Ty, Ix>,
        De: DisplayEdge<N, E, Ty, Ix, Dn>,
    {
        // Only apply layout once
        if self.state.applied {
            return;
        }

        // Collect all nodes with their indices and labels
        let mut nodes: Vec<_> = g
            .nodes_iter()
            .map(|(idx, node)| (idx, node.label().to_string()))
            .collect();

        // Sort according to the configured sort order
        match self.sort_order {
            SortOrder::Alphabetical => {
                nodes.sort_by(|a, b| a.1.cmp(&b.1));
            }
            SortOrder::ReverseAlphabetical => {
                nodes.sort_by(|a, b| b.1.cmp(&a.1));
            }
            SortOrder::None => {
                // Keep insertion order - no sorting
            }
        }

        let node_count = nodes.len();
        if node_count == 0 {
            return;
        }

        // Calculate center of the canvas
        let rect = ui.available_rect_before_wrap();
        let center_x = rect.center().x;
        let center_y = rect.center().y;

        // Calculate radius using configuration
        let radius = if let Some(fixed) = self.spacing.fixed_radius {
            fixed
        } else {
            self.spacing.base_radius + (node_count as f32) * self.spacing.radius_per_node
        };

        // Place nodes in a circle
        for (i, (node_idx, _label)) in nodes.iter().enumerate() {
            // Start at top (-π/2) and go clockwise
            let angle = -std::f32::consts::PI / 2.0
                + (i as f32) * 2.0 * std::f32::consts::PI / (node_count as f32);

            let x = center_x + radius * angle.cos();
            let y = center_y + radius * angle.sin();

            if let Some(node) = g.node_mut(*node_idx) {
                node.set_location(egui::Pos2::new(x, y));
            }
        }

        self.state.applied = true;
    }

    fn state(&self) -> State {
        self.state.clone()
    }
}
