//! Demo-only import specs for layout and extras.
//! This keeps serde DTOs out of the public egui_graphs crate.

use serde::{Deserialize, Serialize};

// Graph schema (reuse minimal form)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GraphSpecMinimal {
    #[serde(default)]
    pub nodes: Vec<i64>,
    #[serde(default)]
    pub edges: Vec<(i64, i64)>,
    #[serde(default)]
    pub directed: Option<bool>,
    #[serde(default)]
    pub positions: Option<Vec<(i64, f32, f32)>>,
}

// Layout and extras specs (only built-ins used in the demo)

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum LayoutSpec {
    FruchtermanReingold {
        #[serde(default)]
        running: Option<bool>,
        #[serde(default)]
        dt: Option<f32>,
        #[serde(default)]
        epsilon: Option<f32>,
        #[serde(default)]
        damping: Option<f32>,
        #[serde(default)]
        max_step: Option<f32>,
        #[serde(default)]
        k_scale: Option<f32>,
        #[serde(default)]
        c_attract: Option<f32>,
        #[serde(default)]
        c_repulse: Option<f32>,
        #[serde(default)]
        extras: Option<Vec<ExtrasSpec>>, // currently only CenterGravity
    },
    Hierarchical {
        #[serde(default)]
        row_dist: Option<f32>,
        #[serde(default)]
        col_dist: Option<f32>,
        #[serde(default)]
        center_parent: Option<bool>,
        #[serde(default)]
        orientation: Option<HierOrientationSpec>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum ExtrasSpec {
    CenterGravity {
        enabled: Option<bool>,
        c: Option<f32>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum HierOrientationSpec {
    TopDown,
    LeftRight,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DemoImportSpec {
    #[serde(default)]
    pub version: Option<u32>,
    #[serde(default)]
    pub graph: Option<GraphSpecMinimal>,
    #[serde(default)]
    pub layout: Option<LayoutSpec>,
}

impl DemoImportSpec {
    pub fn try_parse(text: &str) -> Result<Self, String> {
        serde_json::from_str::<DemoImportSpec>(text)
            .map_err(|e| format!("invalid import spec json: {e}"))
    }
}

// Runtime mapping used by the demo to apply layout later (needs egui UI)
#[derive(Debug, Clone)]
pub enum PendingLayout {
    FR(egui_graphs::FruchtermanReingoldWithCenterGravityState),
    Hier(egui_graphs::LayoutStateHierarchical),
}

impl LayoutSpec {
    pub fn to_pending(&self) -> PendingLayout {
        match self {
            LayoutSpec::FruchtermanReingold {
                running,
                dt,
                epsilon,
                damping,
                max_step,
                k_scale,
                c_attract,
                c_repulse,
                extras,
            } => {
                // Start from defaults and override
                let mut st = egui_graphs::FruchtermanReingoldWithCenterGravityState::default();
                if let Some(v) = running {
                    st.base.is_running = *v;
                }
                if let Some(v) = dt {
                    st.base.dt = *v;
                }
                if let Some(v) = epsilon {
                    st.base.epsilon = *v;
                }
                if let Some(v) = damping {
                    st.base.damping = *v;
                }
                if let Some(v) = max_step {
                    st.base.max_step = *v;
                }
                if let Some(v) = k_scale {
                    st.base.k_scale = *v;
                }
                if let Some(v) = c_attract {
                    st.base.c_attract = *v;
                }
                if let Some(v) = c_repulse {
                    st.base.c_repulse = *v;
                }

                if let Some(list) = extras {
                    for ex in list.iter() {
                        match ex {
                            ExtrasSpec::CenterGravity { enabled, c } => {
                                if let Some(en) = enabled {
                                    st.extras.0.enabled = *en;
                                }
                                if let Some(cv) = c {
                                    st.extras.0.params.c = *cv;
                                }
                            }
                        }
                    }
                }
                PendingLayout::FR(st)
            }
            LayoutSpec::Hierarchical {
                row_dist,
                col_dist,
                center_parent,
                orientation,
            } => {
                let mut st = egui_graphs::LayoutStateHierarchical::default();
                if let Some(v) = row_dist {
                    st.row_dist = *v;
                }
                if let Some(v) = col_dist {
                    st.col_dist = *v;
                }
                if let Some(v) = center_parent {
                    st.center_parent = *v;
                }
                if let Some(o) = orientation {
                    st.orientation = match o {
                        HierOrientationSpec::TopDown => {
                            egui_graphs::LayoutHierarchicalOrientation::TopDown
                        }
                        HierOrientationSpec::LeftRight => {
                            egui_graphs::LayoutHierarchicalOrientation::LeftRight
                        }
                    };
                }
                // Trigger re-run once
                st.triggered = false;
                PendingLayout::Hier(st)
            }
        }
    }
}

// Export helpers (demo-only): read current UI layout state and build LayoutSpec
impl PendingLayout {
    pub fn from_ui_fr_state(ui: &mut egui::Ui) -> LayoutSpec {
        let st = egui_graphs::GraphView::<
            (),
            (),
            petgraph::Directed,
            petgraph::stable_graph::DefaultIx,
            egui_graphs::DefaultNodeShape,
            egui_graphs::DefaultEdgeShape,
            egui_graphs::FruchtermanReingoldWithCenterGravityState,
            egui_graphs::LayoutForceDirected<egui_graphs::FruchtermanReingoldWithCenterGravity>,
        >::get_layout_state(ui);
        LayoutSpec::FruchtermanReingold {
            running: Some(st.base.is_running),
            dt: Some(st.base.dt),
            epsilon: Some(st.base.epsilon),
            damping: Some(st.base.damping),
            max_step: Some(st.base.max_step),
            k_scale: Some(st.base.k_scale),
            c_attract: Some(st.base.c_attract),
            c_repulse: Some(st.base.c_repulse),
            extras: Some(vec![ExtrasSpec::CenterGravity {
                enabled: Some(st.extras.0.enabled),
                c: Some(st.extras.0.params.c),
            }]),
        }
    }

    pub fn from_ui_hier_state(ui: &mut egui::Ui) -> LayoutSpec {
        let st = egui_graphs::GraphView::<
            (),
            (),
            petgraph::Directed,
            petgraph::stable_graph::DefaultIx,
            egui_graphs::DefaultNodeShape,
            egui_graphs::DefaultEdgeShape,
            egui_graphs::LayoutStateHierarchical,
            egui_graphs::LayoutHierarchical,
        >::get_layout_state(ui);
        LayoutSpec::Hierarchical {
            row_dist: Some(st.row_dist),
            col_dist: Some(st.col_dist),
            center_parent: Some(st.center_parent),
            orientation: Some(match st.orientation {
                egui_graphs::LayoutHierarchicalOrientation::TopDown => HierOrientationSpec::TopDown,
                egui_graphs::LayoutHierarchicalOrientation::LeftRight => {
                    HierOrientationSpec::LeftRight
                }
            }),
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub fn build_export_spec(
    ui: &mut egui::Ui,
    include_layout: bool,
    include_graph: bool,
    is_directed: bool,
    selected_layout: crate::DemoLayout,
    g_nodes: Vec<i64>,
    g_edges: Vec<(i64, i64)>,
    g_positions: Option<Vec<(i64, f32, f32)>>,
) -> DemoImportSpec {
    let layout = if include_layout {
        Some(match selected_layout {
            crate::DemoLayout::FruchtermanReingold => PendingLayout::from_ui_fr_state(ui),
            crate::DemoLayout::Hierarchical => PendingLayout::from_ui_hier_state(ui),
        })
    } else {
        None
    };
    let graph = if include_graph {
        Some(GraphSpecMinimal {
            nodes: g_nodes,
            edges: g_edges,
            directed: Some(is_directed),
            positions: g_positions,
        })
    } else {
        None
    };
    DemoImportSpec {
        version: Some(1),
        graph,
        layout,
    }
}
