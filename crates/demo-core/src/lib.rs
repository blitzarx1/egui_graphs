use core::cmp::Ordering;
use eframe::{App, CreationContext};
use egui::{self, CollapsingHeader, Pos2, ScrollArea, Ui};
use egui_graphs::{
    generate_random_graph, FruchtermanReingoldWithCenterGravity,
    FruchtermanReingoldWithCenterGravityState, Graph, LayoutForceDirected, LayoutHierarchical,
    LayoutHierarchicalOrientation, LayoutStateHierarchical,
};
use instant::Instant;
use petgraph::stable_graph::{DefaultIx, EdgeIndex, NodeIndex};
use petgraph::Directed;
use rand::Rng;
#[cfg(all(feature = "events", target_arch = "wasm32"))]
use std::{cell::RefCell, rc::Rc};

mod event_filters;
pub mod info_overlay;
mod keybindings;
mod metrics;

pub const MAX_NODE_COUNT: usize = 2500;
pub const MAX_EDGE_COUNT: usize = 5000;
#[cfg(feature = "events")]
pub const EVENTS_LIMIT: usize = 500;
// Keep margins consistent for overlays/buttons in the CentralPanel
const UI_MARGIN: f32 = 10.0;

#[cfg(feature = "events")]
use crate::event_filters::EventFilters;
use crate::keybindings::{dispatch as dispatch_keybindings, Command};
use crate::metrics::MetricsRecorder;
#[cfg(feature = "events")]
pub use crossbeam::channel::{unbounded, Receiver, Sender};
#[cfg(feature = "events")]
pub use egui_graphs::events::Event;

pub mod settings;

fn info_icon(ui: &mut egui::Ui, tip: &str) {
    ui.add_space(4.0);
    ui.small_button("ℹ").on_hover_text(tip);
}

mod drawers;

pub struct DemoApp {
    pub g: Graph<(), (), Directed, DefaultIx>,
    pub settings_graph: settings::SettingsGraph,
    pub settings_interaction: settings::SettingsInteraction,
    pub settings_navigation: settings::SettingsNavigation,
    pub settings_style: settings::SettingsStyle,
    metrics: MetricsRecorder,
    pub show_sidebar: bool,
    pub dark_mode: bool,
    pub show_debug_overlay: bool,
    pub show_keybindings_overlay: bool,
    pub keybindings_just_opened: bool,
    pub reset_requested: bool,

    // Layout selection for the demo UI
    pub selected_layout: DemoLayout,
    #[cfg(not(feature = "events"))]
    pub copy_tip_until: Option<Instant>,
    #[cfg(feature = "events")]
    pub pan: [f32; 2],
    #[cfg(feature = "events")]
    pub zoom: f32,
    #[cfg(feature = "events")]
    pub last_events: Vec<String>,
    #[cfg(all(feature = "events", not(target_arch = "wasm32")))]
    pub event_publisher: crate::Sender<Event>,
    #[cfg(all(feature = "events", not(target_arch = "wasm32")))]
    pub event_consumer: crate::Receiver<Event>,
    #[cfg(all(feature = "events", target_arch = "wasm32"))]
    pub events_buf: Rc<RefCell<Vec<Event>>>,
    #[cfg(feature = "events")]
    pub event_filters: EventFilters,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DemoLayout {
    FruchtermanReingold,
    Hierarchical,
}
impl DemoApp {
    pub fn new(cc: &CreationContext<'_>) -> Self {
        let settings_graph = settings::SettingsGraph::default();
        let mut g = generate_random_graph(settings_graph.count_node, settings_graph.count_edge);
        Self::distribute_nodes_circle(&mut g);

        #[cfg(all(feature = "events", not(target_arch = "wasm32")))]
        let (event_publisher, event_consumer) = crate::unbounded();
        #[cfg(all(feature = "events", target_arch = "wasm32"))]
        let events_buf: Rc<RefCell<Vec<Event>>> = Rc::new(RefCell::new(Vec::new()));

        Self {
            g,
            settings_graph,
            settings_interaction: settings::SettingsInteraction::default(),
            settings_navigation: settings::SettingsNavigation::default(),
            settings_style: settings::SettingsStyle {
                labels_always: false,
                edge_deemphasis: true,
            },
            metrics: MetricsRecorder::new(),
            // Start with side panel hidden by default
            show_sidebar: false,
            #[cfg(not(feature = "events"))]
            copy_tip_until: None,
            #[cfg(feature = "events")]
            pan: [0.0, 0.0],
            #[cfg(feature = "events")]
            zoom: 1.0,
            #[cfg(feature = "events")]
            last_events: Vec::new(),
            #[cfg(all(feature = "events", not(target_arch = "wasm32")))]
            event_publisher,
            #[cfg(all(feature = "events", not(target_arch = "wasm32")))]
            event_consumer,
            #[cfg(all(feature = "events", target_arch = "wasm32"))]
            events_buf,
            #[cfg(feature = "events")]
            event_filters: EventFilters::default(),
            dark_mode: cc.egui_ctx.style().visuals.dark_mode,
            show_debug_overlay: true,
            show_keybindings_overlay: false,
            keybindings_just_opened: false,
            reset_requested: false,
            selected_layout: DemoLayout::FruchtermanReingold,
        }
    }

    pub fn random_node_idx(&self) -> Option<NodeIndex> {
        let cnt = self.g.node_count();
        if cnt == 0 {
            return None;
        }
        let idx = rand::rng().random_range(0..cnt);
        self.g.g().node_indices().nth(idx)
    }
    pub fn random_edge_idx(&self) -> Option<EdgeIndex> {
        let cnt = self.g.edge_count();
        if cnt == 0 {
            return None;
        }
        let idx = rand::rng().random_range(0..cnt);
        self.g.g().edge_indices().nth(idx)
    }
    pub fn add_random_node(&mut self) {
        if self.g.node_count() >= MAX_NODE_COUNT {
            return;
        }
        let base = if let Some(r) = self.random_node_idx() {
            self.g.node(r).unwrap().location()
        } else {
            Pos2::new(0.0, 0.0)
        };
        let mut rng = rand::rng();
        let loc = Pos2::new(
            base.x + rng.random_range(-150.0..150.0),
            base.y + rng.random_range(-150.0..150.0),
        );
        self.g.add_node_with_location((), loc);
    }
    pub fn remove_random_node(&mut self) {
        if let Some(i) = self.random_node_idx() {
            self.remove_node(i);
        }
    }
    pub fn remove_node(&mut self, idx: NodeIndex) {
        self.g.remove_node(idx);
    }
    pub fn add_random_edge(&mut self) {
        if let (Some(a), Some(b)) = (self.random_node_idx(), self.random_node_idx()) {
            self.add_edge(a, b);
        }
    }
    pub fn add_edge(&mut self, a: NodeIndex, b: NodeIndex) {
        if self.g.edge_count() >= MAX_EDGE_COUNT {
            return;
        }
        self.g.add_edge(a, b, ());
    }
    pub fn remove_random_edge(&mut self) {
        if let Some(eidx) = self.random_edge_idx() {
            if let Some((a, b)) = self.g.edge_endpoints(eidx) {
                self.remove_edge(a, b);
            }
        }
    }
    pub fn remove_edge(&mut self, a: NodeIndex, b: NodeIndex) {
        let edge_id_opt = { self.g.edges_connecting(a, b).map(|(eid, _)| eid).next() };
        if let Some(edge_id) = edge_id_opt {
            self.g.remove_edge(edge_id);
        }
    }

    pub fn update_fps(&mut self) {
        self.metrics.update_fps();
    }

    pub fn ui_graph_section(&mut self, ui: &mut Ui) {
        crate::drawers::graph_count_sliders(
            ui,
            crate::drawers::GraphCountSliders {
                nodes: self.settings_graph.count_node,
                edges: self.settings_graph.count_edge,
            },
            |dn, de| {
                match dn.cmp(&0) {
                    Ordering::Greater => {
                        for _ in 0..dn {
                            self.add_random_node();
                        }
                    }
                    Ordering::Less => {
                        for _ in 0..(-dn) {
                            self.remove_random_node();
                        }
                    }
                    Ordering::Equal => {}
                }
                match de.cmp(&0) {
                    Ordering::Greater => {
                        for _ in 0..de {
                            self.add_random_edge();
                        }
                    }
                    Ordering::Less => {
                        for _ in 0..(-de) {
                            self.remove_random_edge();
                        }
                    }
                    Ordering::Equal => {}
                }
                self.settings_graph.count_node = self.g.node_count();
                self.settings_graph.count_edge = self.g.edge_count();
            },
        );
    }

    pub fn reset_all(&mut self, ui: &mut Ui) {
        self.settings_graph = settings::SettingsGraph::default();
        self.settings_interaction = settings::SettingsInteraction::default();
        self.settings_navigation = settings::SettingsNavigation::default();
        self.settings_style = settings::SettingsStyle {
            labels_always: false,
            edge_deemphasis: true,
        };
        self.show_debug_overlay = true;
        self.show_keybindings_overlay = false;
        self.g = generate_random_graph(
            self.settings_graph.count_node,
            self.settings_graph.count_edge,
        );
        Self::distribute_nodes_circle(&mut self.g);
        egui_graphs::GraphView::<
            (),
            (),
            petgraph::Directed,
            petgraph::stable_graph::DefaultIx,
            egui_graphs::DefaultNodeShape,
            egui_graphs::DefaultEdgeShape,
            FruchtermanReingoldWithCenterGravityState,
            LayoutForceDirected<FruchtermanReingoldWithCenterGravity>,
        >::reset(ui);
        egui_graphs::GraphView::<
            (),
            (),
            petgraph::Directed,
            petgraph::stable_graph::DefaultIx,
            egui_graphs::DefaultNodeShape,
            egui_graphs::DefaultEdgeShape,
            LayoutStateHierarchical,
            LayoutHierarchical,
        >::reset(ui);
        ui.ctx().set_visuals(egui::Visuals::dark());
        self.dark_mode = ui.ctx().style().visuals.dark_mode;
        #[cfg(feature = "events")]
        {
            self.last_events.clear();
            self.pan = [0.0, 0.0];
            self.zoom = 1.0;
            self.event_filters = EventFilters::default();
        }
        self.metrics.reset();
    }

    pub fn distribute_nodes_circle(g: &mut Graph<(), (), Directed, DefaultIx>) {
        let n_usize = core::cmp::max(g.node_count(), 1);
        if n_usize == 0 {
            return;
        }
        let n_f32 = n_usize as f32;
        let radius = n_f32.sqrt() * 50.0 + 50.0;
        let indices: Vec<_> = g.g().node_indices().collect();
        for (i, idx) in indices.into_iter().enumerate() {
            if let Some(node) = g.g_mut().node_weight_mut(idx) {
                let angle = i as f32 / n_f32 * std::f32::consts::TAU;
                node.set_location(Pos2::new(radius * angle.cos(), radius * angle.sin()));
            }
        }
    }

    pub fn ui_navigation(&mut self, ui: &mut Ui) {
        CollapsingHeader::new("Navigation").default_open(true).show(ui, |ui| {
            ui.horizontal(|ui| {
                if ui.checkbox(&mut self.settings_navigation.fit_to_screen_enabled, "fit_to_screen").clicked() {
                    self.settings_navigation.zoom_and_pan_enabled = !self.settings_navigation.zoom_and_pan_enabled;
                }
                info_icon(ui, "Continuously recompute zoom/pan so whole graph stays visible.");
            });
            ui.add_enabled_ui(self.settings_navigation.fit_to_screen_enabled, |ui| {
                ui.horizontal(|ui| {
                    ui.add(egui::Slider::new(&mut self.settings_navigation.fit_to_screen_padding, 0.0..=1.0).text("fit_to_screen_padding"));
                    info_icon(ui, "Extra fractional padding around graph when auto-fitting (0 = tight fit, 0.3 = 30% larger).");
                });
            });
            ui.horizontal(|ui| {
                if ui.checkbox(&mut self.settings_navigation.zoom_and_pan_enabled, "zoom_and_pan").clicked() {
                    self.settings_navigation.fit_to_screen_enabled = !self.settings_navigation.fit_to_screen_enabled;
                }
                info_icon(ui, "Manual navigation: Ctrl+wheel (zoom), drag (pan / node drag). Disable if auto-fit.");
            });
            ui.add_enabled_ui(self.settings_navigation.zoom_and_pan_enabled, |ui| {
                ui.horizontal(|ui| {
                    ui.add(egui::Slider::new(&mut self.settings_navigation.zoom_speed, 0.01..=1.0).text("zoom_speed"));
                    info_icon(ui, "Multiplier controlling how fast zoom changes per wheel step.");
                });
            });
        });
    }

    pub fn ui_layout_section(&mut self, ui: &mut Ui) {
        CollapsingHeader::new("Layout").default_open(true).show(ui, |ui| {
            // Layout selection
            ui.horizontal(|ui| {
                let mut changed = false;
                let r1 = ui.selectable_value(
                    &mut self.selected_layout,
                    DemoLayout::FruchtermanReingold,
                    "Fruchterman-Reingold",
                );
                if r1.changed() { changed = true; }
                let r2 = ui.selectable_value(
                    &mut self.selected_layout,
                    DemoLayout::Hierarchical,
                    "Hierarchical",
                );
                if r2.changed() { changed = true; }

                // If switched to Hierarchical, ensure it recomputes once with current params
                if changed && matches!(self.selected_layout, DemoLayout::Hierarchical) {
                    let mut st = egui_graphs::GraphView::<
                        (), (), petgraph::Directed, petgraph::stable_graph::DefaultIx,
                        egui_graphs::DefaultNodeShape, egui_graphs::DefaultEdgeShape,
                        LayoutStateHierarchical, LayoutHierarchical,
                    >::get_layout_state(ui);
                    st.triggered = false;
                    egui_graphs::GraphView::<
                        (), (), petgraph::Directed, petgraph::stable_graph::DefaultIx,
                        egui_graphs::DefaultNodeShape, egui_graphs::DefaultEdgeShape,
                        LayoutStateHierarchical, LayoutHierarchical,
                    >::set_layout_state(ui, st);
                }
            });

            ui.add_space(6.0);
            // Inline settings for the selected layout
            match self.selected_layout {
                DemoLayout::FruchtermanReingold => {
                    let mut state = egui_graphs::GraphView::<
                        (), (), petgraph::Directed, petgraph::stable_graph::DefaultIx,
                        egui_graphs::DefaultNodeShape, egui_graphs::DefaultEdgeShape,
                        FruchtermanReingoldWithCenterGravityState,
                        LayoutForceDirected<FruchtermanReingoldWithCenterGravity>,
                    >::get_layout_state(ui);

                    ui.horizontal(|ui| {
                        ui.checkbox(&mut state.base.is_running, "running");
                        info_icon(ui, "Run/pause the simulation. When paused node positions stay fixed.");
                    });
                    ui.horizontal(|ui| {
                        ui.add(egui::Slider::new(&mut state.base.dt, 0.001..=0.2).text("dt"));
                        info_icon(ui, "Integration time step (Euler). Larger = faster movement but less stable.");
                    });
                    ui.horizontal(|ui| {
                        ui.add(egui::Slider::new(&mut state.base.damping, 0.0..=1.0).text("damping"));
                        info_icon(ui, "Velocity damping per frame. 1 = no damping, 0 = immediate stop.");
                    });
                    ui.horizontal(|ui| {
                        ui.add(egui::Slider::new(&mut state.base.max_step, 0.1..=50.0).text("max_step"));
                        info_icon(ui, "Maximum pixel displacement applied per frame to prevent explosions.");
                    });
                    ui.horizontal(|ui| {
                        ui.add(egui::Slider::new(&mut state.base.epsilon, 1e-5..=1e-1).logarithmic(true).text("epsilon"));
                        info_icon(ui, "Minimum distance clamp to avoid division by zero in force calculations.");
                    });
                    ui.horizontal(|ui| {
                        ui.add(egui::Slider::new(&mut state.base.k_scale, 0.2..=3.0).text("k_scale"));
                        info_icon(ui, "Scale ideal edge length k; >1 spreads the layout, <1 compacts it.");
                    });
                    ui.horizontal(|ui| {
                        ui.add(egui::Slider::new(&mut state.base.c_attract, 0.1..=3.0).text("c_attract"));
                        info_icon(ui, "Multiplier for attractive force along edges (higher pulls connected nodes together).");
                    });
                    ui.horizontal(|ui| {
                        ui.add(egui::Slider::new(&mut state.base.c_repulse, 0.1..=3.0).text("c_repulse"));
                        info_icon(ui, "Multiplier for repulsive force between nodes (higher pushes nodes apart).");
                    });

                    ui.add_space(6.0);
                    ui.separator();
                    ui.horizontal(|ui| {
                        ui.label("Fast Forward");
                        info_icon(ui, "Advance the simulation instantly by a fixed number of steps or within a frame-time budget.");
                    });
                    ui.vertical(|ui| {
                        if ui.button("Fast-forward 100 steps").clicked() {
                            egui_graphs::GraphView::<
                                (), (), petgraph::Directed, petgraph::stable_graph::DefaultIx,
                                egui_graphs::DefaultNodeShape, egui_graphs::DefaultEdgeShape,
                                FruchtermanReingoldWithCenterGravityState,
                                LayoutForceDirected<FruchtermanReingoldWithCenterGravity>,
                            >::fast_forward_force_run(ui, &mut self.g, 100);
                            state = egui_graphs::GraphView::<
                                (), (), petgraph::Directed, petgraph::stable_graph::DefaultIx,
                                egui_graphs::DefaultNodeShape, egui_graphs::DefaultEdgeShape,
                                FruchtermanReingoldWithCenterGravityState,
                                LayoutForceDirected<FruchtermanReingoldWithCenterGravity>,
                            >::get_layout_state(ui);
                        }
                        if ui.button("Fast-forward 1000 steps_budgeted (100ms)").clicked() {
                            let _done = egui_graphs::GraphView::<
                                (), (), petgraph::Directed, petgraph::stable_graph::DefaultIx,
                                egui_graphs::DefaultNodeShape, egui_graphs::DefaultEdgeShape,
                                FruchtermanReingoldWithCenterGravityState,
                                LayoutForceDirected<FruchtermanReingoldWithCenterGravity>,
                            >::fast_forward_budgeted_force_run(ui, &mut self.g, 1000, 100);
                            state = egui_graphs::GraphView::<
                                (), (), petgraph::Directed, petgraph::stable_graph::DefaultIx,
                                egui_graphs::DefaultNodeShape, egui_graphs::DefaultEdgeShape,
                                FruchtermanReingoldWithCenterGravityState,
                                LayoutForceDirected<FruchtermanReingoldWithCenterGravity>,
                            >::get_layout_state(ui);
                        }
                        if ui.button("Until stable (ε=0.01, ≤1000 steps)").clicked() {
                            let _r = egui_graphs::GraphView::<
                                (), (), petgraph::Directed, petgraph::stable_graph::DefaultIx,
                                egui_graphs::DefaultNodeShape, egui_graphs::DefaultEdgeShape,
                                FruchtermanReingoldWithCenterGravityState,
                                LayoutForceDirected<FruchtermanReingoldWithCenterGravity>,
                            >::fast_forward_until_stable_force_run(ui, &mut self.g, 0.01, 1000);
                            state = egui_graphs::GraphView::<
                                (), (), petgraph::Directed, petgraph::stable_graph::DefaultIx,
                                egui_graphs::DefaultNodeShape, egui_graphs::DefaultEdgeShape,
                                FruchtermanReingoldWithCenterGravityState,
                                LayoutForceDirected<FruchtermanReingoldWithCenterGravity>,
                            >::get_layout_state(ui);
                        }
                        if ui.button("Until stable_budgeted (ε=0.01, ≤10000 steps, 1000ms)").clicked() {
                            let _r = egui_graphs::GraphView::<
                                (), (), petgraph::Directed, petgraph::stable_graph::DefaultIx,
                                egui_graphs::DefaultNodeShape, egui_graphs::DefaultEdgeShape,
                                FruchtermanReingoldWithCenterGravityState,
                                LayoutForceDirected<FruchtermanReingoldWithCenterGravity>,
                            >::fast_forward_until_stable_budgeted_force_run(ui, &mut self.g, 0.01, 10000, 1000);
                            state = egui_graphs::GraphView::<
                                (), (), petgraph::Directed, petgraph::stable_graph::DefaultIx,
                                egui_graphs::DefaultNodeShape, egui_graphs::DefaultEdgeShape,
                                FruchtermanReingoldWithCenterGravityState,
                                LayoutForceDirected<FruchtermanReingoldWithCenterGravity>,
                            >::get_layout_state(ui);
                        }
                    });

                    ui.add_space(6.0);
                    ui.separator();
                    ui.label("Extras");
                    ui.horizontal(|ui| {
                        ui.checkbox(&mut state.extras.0.enabled, "center_gravity");
                        info_icon(ui, "Enable/disable center gravity force.");
                    });
                    ui.add_enabled_ui(state.extras.0.enabled, |ui| {
                        ui.horizontal(|ui| {
                            ui.add(egui::Slider::new(&mut state.extras.0.params.c, 0.0..=2.0).text("center_strength"));
                            info_icon(ui, "Coefficient for pull toward viewport/graph center.");
                        });
                    });

                    egui_graphs::GraphView::<
                        (), (), petgraph::Directed, petgraph::stable_graph::DefaultIx,
                        egui_graphs::DefaultNodeShape, egui_graphs::DefaultEdgeShape,
                        FruchtermanReingoldWithCenterGravityState,
                        LayoutForceDirected<FruchtermanReingoldWithCenterGravity>,
                    >::set_layout_state(ui, state);
                }
                DemoLayout::Hierarchical => {
                    let mut state = egui_graphs::GraphView::<
                        (), (), petgraph::Directed, petgraph::stable_graph::DefaultIx,
                        egui_graphs::DefaultNodeShape, egui_graphs::DefaultEdgeShape,
                        LayoutStateHierarchical, LayoutHierarchical,
                    >::get_layout_state(ui);

                    ui.horizontal(|ui| {
                        ui.add(egui::Slider::new(&mut state.row_dist, 10.0..=500.0).text("row_dist"));
                        info_icon(ui, "Distance between levels (rows). For LeftRight, this is X step.");
                    });
                    ui.horizontal(|ui| {
                        ui.add(egui::Slider::new(&mut state.col_dist, 10.0..=800.0).text("col_dist"));
                        info_icon(ui, "Distance between siblings (columns). For LeftRight, this is Y step.");
                    });
                    ui.horizontal(|ui| {
                        ui.checkbox(&mut state.center_parent, "center_parent");
                        info_icon(ui, "Center parent above/beside the span of its children.");
                    });
                    ui.horizontal(|ui| {
                        ui.selectable_value(&mut state.orientation, LayoutHierarchicalOrientation::TopDown, "TopDown");
                        ui.selectable_value(&mut state.orientation, LayoutHierarchicalOrientation::LeftRight, "LeftRight");
                    });
                    ui.horizontal(|ui| {
                        if ui.button("Re-run layout").clicked() {
                            state.triggered = false;
                        }
                        info_icon(ui, "Apply updated parameters and recompute positions once.");
                    });

                    egui_graphs::GraphView::<
                        (), (), petgraph::Directed, petgraph::stable_graph::DefaultIx,
                        egui_graphs::DefaultNodeShape, egui_graphs::DefaultEdgeShape,
                        LayoutStateHierarchical, LayoutHierarchical,
                    >::set_layout_state(ui, state);
                }
            }
        });
    }

    pub fn ui_layout_force_directed(&mut self, ui: &mut Ui) {
        let state = egui_graphs::GraphView::<
            (),
            (),
            petgraph::Directed,
            petgraph::stable_graph::DefaultIx,
            egui_graphs::DefaultNodeShape,
            egui_graphs::DefaultEdgeShape,
            FruchtermanReingoldWithCenterGravityState,
            LayoutForceDirected<FruchtermanReingoldWithCenterGravity>,
        >::get_layout_state(ui);

        egui_graphs::GraphView::<
            (),
            (),
            petgraph::Directed,
            petgraph::stable_graph::DefaultIx,
            egui_graphs::DefaultNodeShape,
            egui_graphs::DefaultEdgeShape,
            FruchtermanReingoldWithCenterGravityState,
            LayoutForceDirected<FruchtermanReingoldWithCenterGravity>,
        >::set_layout_state(ui, state);
    }

    pub fn ui_layout_hierarchical(&mut self, ui: &mut Ui) {
        let mut state = egui_graphs::GraphView::<
            (),
            (),
            petgraph::Directed,
            petgraph::stable_graph::DefaultIx,
            egui_graphs::DefaultNodeShape,
            egui_graphs::DefaultEdgeShape,
            LayoutStateHierarchical,
            LayoutHierarchical,
        >::get_layout_state(ui);

        CollapsingHeader::new("Hierarchical Layout")
            .default_open(true)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.add(egui::Slider::new(&mut state.row_dist, 10.0..=500.0).text("row_dist"));
                    info_icon(
                        ui,
                        "Distance between levels (rows). For LeftRight, this is X step.",
                    );
                });
                ui.horizontal(|ui| {
                    ui.add(egui::Slider::new(&mut state.col_dist, 10.0..=800.0).text("col_dist"));
                    info_icon(
                        ui,
                        "Distance between siblings (columns). For LeftRight, this is Y step.",
                    );
                });
                ui.horizontal(|ui| {
                    ui.checkbox(&mut state.center_parent, "center_parent");
                    info_icon(ui, "Center parent above/beside the span of its children.");
                });
                ui.horizontal(|ui| {
                    ui.label("orientation");
                    let mut o = state.orientation;
                    if ui
                        .selectable_value(&mut o, LayoutHierarchicalOrientation::TopDown, "TopDown")
                        .clicked()
                    {
                        state.orientation = o;
                    }
                    if ui
                        .selectable_value(
                            &mut o,
                            LayoutHierarchicalOrientation::LeftRight,
                            "LeftRight",
                        )
                        .clicked()
                    {
                        state.orientation = o;
                    }
                });

                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    if ui.button("Re-run layout").clicked() {
                        state.triggered = false;
                    }
                    info_icon(ui, "Apply updated parameters and recompute positions once.");
                });
            });

        egui_graphs::GraphView::<
            (),
            (),
            petgraph::Directed,
            petgraph::stable_graph::DefaultIx,
            egui_graphs::DefaultNodeShape,
            egui_graphs::DefaultEdgeShape,
            LayoutStateHierarchical,
            LayoutHierarchical,
        >::set_layout_state(ui, state);
    }

    pub fn ui_interaction(&mut self, ui: &mut Ui) {
        CollapsingHeader::new("Interaction").show(ui, |ui| {
            ui.horizontal(|ui| {
                if ui.checkbox(&mut self.settings_interaction.dragging_enabled, "dragging_enabled").clicked()
                    && self.settings_interaction.dragging_enabled
                {
                    self.settings_interaction.node_clicking_enabled = true;
                    self.settings_interaction.hover_enabled = true;
                }
                info_icon(ui, "Master: also enables node_clicking and hover.");
            });
            ui.add_enabled_ui(
                !self.settings_interaction.dragging_enabled
                    && !self.settings_interaction.node_selection_enabled
                    && !self.settings_interaction.node_selection_multi_enabled
                    && !self.settings_interaction.edge_selection_enabled
                    && !self.settings_interaction.edge_selection_multi_enabled,
                |ui| {
                    ui.horizontal(|ui| {
                        ui.checkbox(&mut self.settings_interaction.hover_enabled, "hover_enabled");
                        info_icon(ui, "Disabled while any master is enabled (dragging/selection/multiselection).");
                    });
                },
            );
            ui.add_enabled_ui(
                !self.settings_interaction.dragging_enabled
                    && !self.settings_interaction.node_selection_enabled
                    && !self.settings_interaction.node_selection_multi_enabled
                    && !self.settings_interaction.edge_selection_enabled
                    && !self.settings_interaction.edge_selection_multi_enabled,
                |ui| {
                    ui.horizontal(|ui| {
                        ui.checkbox(&mut self.settings_interaction.node_clicking_enabled, "node_clicking");
                        info_icon(ui, "Disabled while any master is enabled (dragging/selection/multiselection).");
                    });
                },
            );
            ui.add_enabled_ui(!self.settings_interaction.node_selection_multi_enabled, |ui| {
                ui.horizontal(|ui| {
                    if ui.checkbox(&mut self.settings_interaction.node_selection_enabled, "node_selection").clicked()
                        && self.settings_interaction.node_selection_enabled
                    {
                        self.settings_interaction.node_clicking_enabled = true;
                        self.settings_interaction.hover_enabled = true;
                    }
                    info_icon(ui, "Master: also enables node_clicking and hover.");
                });
            });
            ui.horizontal(|ui| {
                if ui
                    .checkbox(
                        &mut self.settings_interaction.node_selection_multi_enabled,
                        "node_selection_multi",
                    )
                    .changed()
                    && self.settings_interaction.node_selection_multi_enabled
                {
                    self.settings_interaction.node_selection_enabled = true;
                    self.settings_interaction.node_clicking_enabled = true;
                    self.settings_interaction.hover_enabled = true;
                }
                info_icon(ui, "Master: also enables selection, node_clicking and hover.");
            });
            ui.add_enabled_ui(
                !(self.settings_interaction.edge_selection_enabled
                    || self.settings_interaction.edge_selection_multi_enabled),
                |ui| {
                    ui.horizontal(|ui| {
                        ui.add_enabled_ui(
                            !self.settings_interaction.edge_selection_enabled
                                && !self.settings_interaction.edge_selection_multi_enabled
                                && !self.settings_interaction.dragging_enabled
                                && !self.settings_interaction.node_selection_enabled
                                && !self.settings_interaction.node_selection_multi_enabled,
                            |ui| {
                                ui.checkbox(&mut self.settings_interaction.edge_clicking_enabled, "edge_clicking");
                            },
                        );
                        info_icon(ui, "Disabled while any master is enabled (dragging/selection/multiselection).");
                    });
                },
            );
            ui.add_enabled_ui(!self.settings_interaction.edge_selection_multi_enabled, |ui| {
                ui.horizontal(|ui| {
                    if ui.checkbox(&mut self.settings_interaction.edge_selection_enabled, "edge_selection").clicked()
                        && self.settings_interaction.edge_selection_enabled
                    {
                        self.settings_interaction.edge_clicking_enabled = true;
                        self.settings_interaction.hover_enabled = true;
                    }
                    info_icon(ui, "Master: also enables node_clicking and hover.");
                });
            });
            ui.horizontal(|ui| {
                if ui
                    .checkbox(
                        &mut self.settings_interaction.edge_selection_multi_enabled,
                        "edge_selection_multi",
                    )
                    .changed()
                    && self.settings_interaction.edge_selection_multi_enabled
                {
                    self.settings_interaction.edge_selection_enabled = true;
                    self.settings_interaction.edge_clicking_enabled = true;
                    self.settings_interaction.hover_enabled = true;
                }
                info_icon(ui, "Master: also enables selection, node_clicking and hover.");
            });
        });
    }

    pub fn ui_style(&mut self, ui: &mut Ui) {
        CollapsingHeader::new("Style").show(ui, |ui| {
            ui.horizontal(|ui| {
                let mut dark = ui.ctx().style().visuals.dark_mode;
                if ui
                    .checkbox(&mut dark, "dark mode")
                    .on_hover_text("Toggle dark or light visuals")
                    .changed()
                {
                    if dark {
                        ui.ctx().set_visuals(egui::Visuals::dark());
                    } else {
                        ui.ctx().set_visuals(egui::Visuals::light());
                    }
                    self.dark_mode = dark;
                } else {
                    self.dark_mode = dark;
                }
                info_icon(
                    ui,
                    "Synced with global egui style context for seamless integration.",
                );
            });
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.checkbox(&mut self.settings_style.labels_always, "labels_always");
                info_icon(
                    ui,
                    "Always render node & edge labels instead of only on interaction.",
                );
            });
            ui.add_space(2.0);
            ui.horizontal(|ui| {
                ui.checkbox(&mut self.settings_style.edge_deemphasis, "edge_deemphasis");
                info_icon(ui, "Dim non-selected edges to highlight current selection.");
            });
        });
    }

    pub fn ui_debug(&mut self, ui: &mut Ui) {
        CollapsingHeader::new("Debug")
            .default_open(false)
            .show(ui, |ui| {
                if ui
                    .checkbox(&mut self.show_debug_overlay, "show debug overlay")
                    .on_hover_text("Toggle debug overlay (d)")
                    .clicked()
                {
                    // nothing extra
                }
                if ui
                    .button("keybindings")
                    .on_hover_text("Show keybindings (h / ?)")
                    .clicked()
                {
                    self.show_keybindings_overlay = true;
                    self.keybindings_just_opened = true;
                }
            });
    }

    pub fn ui_selected(&mut self, ui: &mut Ui) {
        CollapsingHeader::new("Selected")
            .default_open(true)
            .show(ui, |ui| {
                ScrollArea::vertical().max_height(150.0).show(ui, |ui| {
                    for n in self.g.selected_nodes() {
                        ui.label(format!("{n:?}"));
                    }
                    for e in self.g.selected_edges() {
                        ui.label(format!("{e:?}"));
                    }
                });
            });
    }

    #[cfg(feature = "events")]
    pub fn ui_events(&mut self, ui: &mut Ui) {
        CollapsingHeader::new("Events")
            .default_open(true)
            .show(ui, |ui| {
                // Ensure the events section has a reasonable minimum height so it doesn't collapse too small.
                ui.set_min_height(220.0);
                ui.horizontal(|ui| {
                    if ui.button("All").clicked() {
                        self.event_filters = EventFilters {
                            pan: true,
                            zoom: true,
                            node_move: true,
                            node_drag_start: true,
                            node_drag_end: true,
                            node_hover_enter: true,
                            node_hover_leave: true,
                            node_select: true,
                            node_deselect: true,
                            node_click: true,
                            node_double_click: true,
                            edge_click: true,
                            edge_select: true,
                            edge_deselect: true,
                        };
                    }
                    if ui.button("None").clicked() {
                        self.event_filters = EventFilters {
                            pan: false,
                            zoom: false,
                            node_move: false,
                            node_drag_start: false,
                            node_drag_end: false,
                            node_hover_enter: false,
                            node_hover_leave: false,
                            node_select: false,
                            node_deselect: false,
                            node_click: false,
                            node_double_click: false,
                            edge_click: false,
                            edge_select: false,
                            edge_deselect: false,
                        };
                        // After disabling all, clear list for clarity
                        self.last_events.clear();
                    }
                    if ui.button("Clear").clicked() {
                        self.last_events.clear();
                    }
                    ui.label(format!(
                        "showing {} / {}",
                        self.last_events.len(),
                        EVENTS_LIMIT
                    ));
                });

                ui.separator();
                ui.label("Filters");
                egui::Grid::new("events_filters_grid")
                    .num_columns(2)
                    .spacing(egui::vec2(12.0, 4.0))
                    .show(ui, |ui| {
                        let mut changed = false;
                        changed |= ui.checkbox(&mut self.event_filters.pan, "Pan").changed();
                        changed |= ui.checkbox(&mut self.event_filters.zoom, "Zoom").changed();
                        ui.end_row();
                        changed |= ui
                            .checkbox(&mut self.event_filters.node_move, "NodeMove")
                            .changed();
                        changed |= ui
                            .checkbox(&mut self.event_filters.node_drag_start, "NodeDragStart")
                            .changed();
                        ui.end_row();
                        changed |= ui
                            .checkbox(&mut self.event_filters.node_drag_end, "NodeDragEnd")
                            .changed();
                        changed |= ui
                            .checkbox(&mut self.event_filters.node_hover_enter, "NodeHoverEnter")
                            .changed();
                        ui.end_row();
                        changed |= ui
                            .checkbox(&mut self.event_filters.node_hover_leave, "NodeHoverLeave")
                            .changed();
                        changed |= ui
                            .checkbox(&mut self.event_filters.node_select, "NodeSelect")
                            .changed();
                        ui.end_row();
                        changed |= ui
                            .checkbox(&mut self.event_filters.node_deselect, "NodeDeselect")
                            .changed();
                        changed |= ui
                            .checkbox(&mut self.event_filters.node_click, "NodeClick")
                            .changed();
                        ui.end_row();
                        changed |= ui
                            .checkbox(&mut self.event_filters.node_double_click, "NodeDoubleClick")
                            .changed();
                        changed |= ui
                            .checkbox(&mut self.event_filters.edge_click, "EdgeClick")
                            .changed();
                        ui.end_row();
                        changed |= ui
                            .checkbox(&mut self.event_filters.edge_select, "EdgeSelect")
                            .changed();
                        changed |= ui
                            .checkbox(&mut self.event_filters.edge_deselect, "EdgeDeselect")
                            .changed();
                        ui.end_row();

                        if changed {
                            // Drop already stored events that are no longer enabled
                            self.event_filters.purge_disabled(&mut self.last_events);
                            ui.ctx().request_repaint();
                        }
                    });

                ui.separator();
                let list_h = ui.available_height();
                ScrollArea::vertical().max_height(list_h).show(ui, |ui| {
                    // Show in chronological order; newest at bottom.
                    for ev in &self.last_events {
                        ui.code(ev);
                    }
                });
            });
    }

    #[cfg(not(feature = "events"))]
    pub fn ui_events(&mut self, ui: &mut Ui) {
        self.show_events_feature_tip(ui);
    }

    #[cfg(not(feature = "events"))]
    pub fn show_events_feature_tip(&mut self, ui: &mut Ui) {
        ui.group(|ui| {
            ui.colored_label(egui::Color32::from_rgb(200, 180, 40),
                "Tip: enable the 'events' feature to see interaction events (pan/zoom, clicks, selections).",
            );
        });
    }
    #[cfg(feature = "events")]
    pub fn show_events_feature_tip(&mut self, _ui: &mut Ui) {}

    pub fn sync_counts(&mut self) {
        self.settings_graph.count_node = self.g.node_count();
        self.settings_graph.count_edge = self.g.edge_count();
    }
}

impl App for DemoApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Sync counts displayed on sliders with actual graph values
        self.sync_counts();

        // Handle global keyboard shortcuts and modal toggling
        self.process_keybindings(ctx);

        // Right side panel with controls
        if self.show_sidebar {
            egui::SidePanel::right("right")
                .default_width(300.0)
                .min_width(300.0)
                .show(ctx, |ui| {
                    // Single scroll area: all sections scroll naturally; Events is last.
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        if ui
                            .button("Reset Defaults")
                            .on_hover_text("Reset ALL settings, graph, layout & view state (Space)")
                            .clicked()
                        {
                            self.reset_all(ui);
                        }
                        CollapsingHeader::new("Graph / Layout")
                            .default_open(true)
                            .show(ui, |ui| self.ui_graph_section(ui));
                        self.ui_navigation(ui);
                        self.ui_layout_section(ui);
                        self.ui_layout_force_directed(ui);
                        self.ui_interaction(ui);
                        // Selected under Interaction
                        self.ui_selected(ui);
                        self.ui_style(ui);
                        self.ui_debug(ui);
                        // Events or tip shown last and scrollable
                        self.ui_events(ui);
                    });
                });
        }

        // Central graph view
        egui::CentralPanel::default().show(ctx, |ui| {
            if self.reset_requested {
                self.reset_all(ui);
                self.reset_requested = false;
            }
            let settings_interaction = &egui_graphs::SettingsInteraction::new()
                .with_node_selection_enabled(self.settings_interaction.node_selection_enabled)
                .with_node_selection_multi_enabled(
                    self.settings_interaction.node_selection_multi_enabled,
                )
                .with_dragging_enabled(self.settings_interaction.dragging_enabled)
                .with_hover_enabled(self.settings_interaction.hover_enabled)
                .with_node_clicking_enabled(self.settings_interaction.node_clicking_enabled)
                .with_edge_clicking_enabled(self.settings_interaction.edge_clicking_enabled)
                .with_edge_selection_enabled(self.settings_interaction.edge_selection_enabled)
                .with_edge_selection_multi_enabled(
                    self.settings_interaction.edge_selection_multi_enabled,
                );
            let settings_navigation = &egui_graphs::SettingsNavigation::new()
                .with_zoom_and_pan_enabled(self.settings_navigation.zoom_and_pan_enabled)
                .with_fit_to_screen_enabled(self.settings_navigation.fit_to_screen_enabled)
                .with_zoom_speed(self.settings_navigation.zoom_speed)
                .with_fit_to_screen_padding(self.settings_navigation.fit_to_screen_padding);
            let mut style_builder = egui_graphs::SettingsStyle::new()
                .with_labels_always(self.settings_style.labels_always);
            if self.settings_style.edge_deemphasis {
                style_builder =
                    style_builder.with_edge_stroke_hook(|selected, _order, stroke, _style| {
                        let mut s = stroke;
                        if !selected {
                            let c = s.color;
                            let new_a = (f32::from(c.a()) * 0.5) as u8;
                            s.color =
                                egui::Color32::from_rgba_unmultiplied(c.r(), c.g(), c.b(), new_a);
                        }
                        s
                    });
            }
            let settings_style = &style_builder;

            match self.selected_layout {
                DemoLayout::FruchtermanReingold => {
                    let mut view = egui_graphs::GraphView::<
                        _,
                        _,
                        _,
                        _,
                        _,
                        _,
                        FruchtermanReingoldWithCenterGravityState,
                        LayoutForceDirected<FruchtermanReingoldWithCenterGravity>,
                    >::new(&mut self.g)
                    .with_interactions(settings_interaction)
                    .with_navigations(settings_navigation)
                    .with_styles(settings_style);
                    #[cfg(feature = "events")]
                    {
                        #[cfg(not(target_arch = "wasm32"))]
                        {
                            view = view.with_event_sink(&self.event_publisher);
                        }
                        #[cfg(target_arch = "wasm32")]
                        {
                            view = view.with_event_sink(&self.events_buf);
                        }
                    }
                    ui.add(&mut view);
                }
                DemoLayout::Hierarchical => {
                    let mut view = egui_graphs::GraphView::<
                        _,
                        _,
                        _,
                        _,
                        _,
                        _,
                        LayoutStateHierarchical,
                        LayoutHierarchical,
                    >::new(&mut self.g)
                    .with_interactions(settings_interaction)
                    .with_navigations(settings_navigation)
                    .with_styles(settings_style);
                    #[cfg(feature = "events")]
                    {
                        #[cfg(not(target_arch = "wasm32"))]
                        {
                            view = view.with_event_sink(&self.event_publisher);
                        }
                        #[cfg(target_arch = "wasm32")]
                        {
                            view = view.with_event_sink(&self.events_buf);
                        }
                    }
                    ui.add(&mut view);
                }
            }

            #[cfg(feature = "events")]
            self.consume_events();

            // Capture latest layout step count for overlay display
            if let DemoLayout::FruchtermanReingold = self.selected_layout {
                let state = egui_graphs::GraphView::<
                    (),
                    (),
                    petgraph::Directed,
                    petgraph::stable_graph::DefaultIx,
                    egui_graphs::DefaultNodeShape,
                    egui_graphs::DefaultEdgeShape,
                    FruchtermanReingoldWithCenterGravityState,
                    LayoutForceDirected<FruchtermanReingoldWithCenterGravity>,
                >::get_layout_state(ui);
                self.metrics
                    .set_last_step_count(state.base.step_count as usize);
            } else {
                self.metrics.set_last_step_count(0);
            }

            // Record performance samples for 5s rolling average
            self.record_perf_sample(ui);
            // Small info overlay (top-left): version + source link
            crate::info_overlay::render_info_overlay(ui);
            // Draw overlay inside the CentralPanel so it stays within the graph area
            self.overlay_debug_panel(ui);
            // Draw toggle button for the side panel at bottom-right of the graph area
            self.overlay_toggle_sidebar_button(ui);
            // Bottom tips removed; use single-line version/source overlay instead (moved below)
        });

        self.update_fps();

        // Draw modal after main UI
        self.keybindings_modal(ctx);
    }
}

impl DemoApp {
    fn record_perf_sample(&mut self, ui: &mut egui::Ui) {
        let (step_ms, draw_ms) = match self.selected_layout {
            DemoLayout::FruchtermanReingold => egui_graphs::GraphView::<
                (),
                (),
                petgraph::Directed,
                petgraph::stable_graph::DefaultIx,
                egui_graphs::DefaultNodeShape,
                egui_graphs::DefaultEdgeShape,
                FruchtermanReingoldWithCenterGravityState,
                LayoutForceDirected<FruchtermanReingoldWithCenterGravity>,
            >::get_metrics(ui),
            DemoLayout::Hierarchical => egui_graphs::GraphView::<
                (),
                (),
                petgraph::Directed,
                petgraph::stable_graph::DefaultIx,
                egui_graphs::DefaultNodeShape,
                egui_graphs::DefaultEdgeShape,
                LayoutStateHierarchical,
                LayoutHierarchical,
            >::get_metrics(ui),
        };

        self.metrics.record_sample(step_ms, draw_ms);
    }
    #[cfg(feature = "events")]
    fn consume_events(&mut self) {
        use egui_graphs::events::Event;
        let mut push_event = |e: &Event| {
            if !self.event_filters.enabled_for(e) {
                return;
            }
            match e {
                Event::Pan(p) => self.pan = p.new_pan,
                Event::Zoom(z) => self.zoom = z.new_zoom,
                _ => {}
            }
            let s = format!("{:?}", e);
            self.last_events.push(s);
            if self.last_events.len() > crate::EVENTS_LIMIT {
                let overflow = self.last_events.len() - crate::EVENTS_LIMIT;
                self.last_events.drain(0..overflow);
            }
        };

        #[cfg(not(target_arch = "wasm32"))]
        {
            while let Ok(e) = self.event_consumer.try_recv() {
                push_event(&e);
            }
        }
        #[cfg(target_arch = "wasm32")]
        {
            let mut buf = self.events_buf.borrow_mut();
            for e in buf.drain(..) {
                push_event(&e);
            }
        }
    }
    fn overlay_debug_panel(&mut self, ui: &mut egui::Ui) {
        if !self.show_debug_overlay {
            return;
        }

        // Compose overlay text
        let text = {
            let fps_line = format!("FPS: {:.1}", self.metrics.fps());
            // Averages over last 5 seconds
            let step_avg = self.metrics.step_avg_5s();
            let draw_avg = self.metrics.draw_avg_5s();
            let step_line = format!("TStep: {:.2} ms (avg 5s)", step_avg);
            let draw_line = format!("TDraw: {:.2} ms (avg 5s)", draw_avg);
            let node_count = self.g.node_count();
            let edge_count = self.g.edge_count();
            let n_line = if node_count >= MAX_NODE_COUNT {
                format!("N: {node_count} MAX")
            } else {
                format!("N: {node_count}")
            };
            let e_line = if edge_count >= MAX_EDGE_COUNT {
                format!("E: {edge_count} MAX")
            } else {
                format!("E: {edge_count}")
            };
            let steps_line = format!("Steps: {}", self.metrics.last_step_count());
            #[cfg(feature = "events")]
            let zoom_line = if self.event_filters.zoom {
                format!("Zoom: {:.3}", self.zoom)
            } else {
                "Zoom: (filter off)".to_string()
            };
            #[cfg(feature = "events")]
            let pan_line = if self.event_filters.pan {
                format!("Pan: [{:.1},{:.1}]", self.pan[0], self.pan[1])
            } else {
                "Pan: (filter off)".to_string()
            };

            #[cfg(feature = "events")]
            {
                format!("{fps_line}\n{step_line}\n{draw_line}\n{n_line}\n{e_line}\n{steps_line}\n{zoom_line}\n{pan_line}")
            }
            #[cfg(not(feature = "events"))]
            {
                format!(
                "{fps_line}\n{step_line}\n{draw_line}\n{n_line}\n{e_line}\n{steps_line}\nZoom: enable events feature\nPan: enable events feature"
            )
            }
        };

        let text_color = ui.style().visuals.strong_text_color();
        let panel_rect = ui.max_rect();
        let font_id = egui::FontId::monospace(14.0);
        // Layout without wrapping: each line stays single-line
        let galley = ui.fonts(|f| f.layout_no_wrap(text.clone(), font_id, text_color));
        // Position galley at top-right with margin
        let pos = egui::pos2(
            panel_rect.right() - UI_MARGIN - galley.size().x,
            panel_rect.top() + UI_MARGIN,
        );
        // Paint within the CentralPanel clip rect to keep it inside
        let painter = ui.painter_at(panel_rect);
        painter.galley(pos, galley, text_color);
    }

    fn keybindings_modal(&mut self, ctx: &egui::Context) {
        // Use egui::Modal so it renders above overlays and dims the background.
        let modal = egui::Modal::new(egui::Id::new("keybindings_modal"));
        if self.show_keybindings_overlay {
            modal.show(ctx, |ui| {
            use egui::RichText;
            let accent = ui.visuals().hyperlink_color;
            ui.label(RichText::new("Keybindings").strong().size(20.0).color(accent));
            ui.separator();

            let render_group = |ui: &mut egui::Ui, title: &str, entries: &[(&str, &str)], grid_id: &str| {
                ui.label(RichText::new(title).strong().color(accent).size(16.0));
                egui::Grid::new(grid_id).num_columns(2).spacing(egui::vec2(8.0, 4.0)).show(ui, |ui| {
                    for (key, desc) in entries {
                        ui.code(*key);
                        ui.label(*desc);
                        ui.end_row();
                    }
                });
                ui.add_space(6.0);
            };

            // Graph elements
            render_group(ui, "Graph elements",
                &[
                    ("n", "add 1 node"),
                    ("Shift+n", "remove 1 node"),
                    ("Ctrl+Shift+n", "swap 1 node (remove then add)"),
                    ("m", "+10 nodes (up to max)"),
                    ("Shift+m", "-10 nodes"),
                    ("Ctrl+Shift+m", "swap 10 nodes"),
                    ("e", "add 1 edge"),
                    ("Shift+e", "remove 1 edge"),
                    ("Ctrl+Shift+e", "swap 1 edge"),
                    ("r", "+10 edges (up to max)"),
                    ("Shift+r", "-10 edges"),
                    ("Ctrl+Shift+r", "swap 10 edges"),
                ],
                "kb_group_elements");

            // Graph actions
            render_group(ui, "Graph actions",
                &[
                    ("d", "toggle debug overlay"),
                    ("Space", "reset all"),
                ],
                "kb_group_actions");

            // Interface / navigation (notes about enabling the corresponding settings)
            render_group(ui, "Interface",
                &[
                    ("Drag", "move nodes (requires: dragging_enabled)"),
                    ("Click", "select node/edge (requires: node_clicking / node_selection / edge_selection)"),
                    ("Ctrl+Wheel", "zoom (requires: zoom_and_pan)"),
                    ("Drag background", "pan (requires: zoom_and_pan)"),
                    ("Tab", "toggle right side panel"),
                    ("h / ?", "open keybindings (this) modal"),
                ],
                "kb_group_interface");
    });
        }
    }

    fn overlay_toggle_sidebar_button(&mut self, ui: &mut egui::Ui) {
        // Small overlay buttons inside the CentralPanel: '?' (help) and '<'/'>' (toggle sidebar)
        let g_rect = ui.max_rect();
        let btn_size = egui::vec2(28.0, 28.0);
        let spacing = 6.0;
        // Use the same external padding as the debug overlay
        let right_margin = UI_MARGIN;
        let bottom_margin = UI_MARGIN;
        let toggle_pos = egui::pos2(
            g_rect.right() - right_margin - btn_size.x,
            g_rect.bottom() - bottom_margin - btn_size.y,
        );
        let help_pos = egui::pos2(toggle_pos.x - spacing - btn_size.x, toggle_pos.y);

        // Use filled triangles for a nicer look
        let (arrow, tip) = if self.show_sidebar {
            ("▶", "Hide side panel (Tab)")
        } else {
            ("◀", "Show side panel (Tab)")
        };

        // Help '?' button (opens keybindings)
        egui::Area::new(egui::Id::new("help_btn"))
            .order(egui::Order::Middle)
            .fixed_pos(help_pos)
            .movable(false)
            .show(ui.ctx(), |ui_area| {
                // Clip the button to the CentralPanel rect
                ui_area.set_clip_rect(g_rect);
                let help_text = egui::RichText::new("ℹ").size(16.0);
                let response = ui_area.add_sized(btn_size, egui::Button::new(help_text));
                if response.on_hover_text("Open keybindings (h / ?)").clicked() {
                    self.show_keybindings_overlay = true;
                    self.keybindings_just_opened = true;
                }
            });

        // Sidebar toggle button
        egui::Area::new(egui::Id::new("sidebar_toggle_btn"))
            .order(egui::Order::Middle)
            .fixed_pos(toggle_pos)
            .movable(false)
            .show(ui.ctx(), |ui_area| {
                // Clip the button to the CentralPanel rect
                ui_area.set_clip_rect(g_rect);
                let arrow_text = egui::RichText::new(arrow).size(16.0);
                let response = ui_area.add_sized(btn_size, egui::Button::new(arrow_text));
                if response.on_hover_text(tip).clicked() {
                    self.show_sidebar = !self.show_sidebar;
                }
            });
    }
    // Bottom instructional tips removed
    fn process_keybindings(&mut self, ctx: &egui::Context) {
        let cmds = dispatch_keybindings(ctx);
        let mut open_modal = false;
        let mut close_modal = false;
        for c in cmds {
            match c {
                Command::ToggleSidebar => self.show_sidebar = !self.show_sidebar,
                Command::ToggleDebug => self.show_debug_overlay = !self.show_debug_overlay,
                Command::OpenKeybindings => {
                    if self.show_keybindings_overlay {
                        close_modal = true;
                    } else {
                        open_modal = true;
                    }
                }
                Command::CloseKeybindings => self.show_keybindings_overlay = false,
                Command::ResetAll => self.reset_requested = true,
                Command::AddNodes(n) => {
                    for _ in 0..n {
                        self.add_random_node();
                    }
                }
                Command::RemoveNodes(n) => {
                    for _ in 0..n {
                        self.remove_random_node();
                    }
                }
                Command::SwapNodes(n) => {
                    for _ in 0..n {
                        self.remove_random_node();
                        self.add_random_node();
                    }
                }
                Command::AddEdges(n) => {
                    for _ in 0..n {
                        self.add_random_edge();
                    }
                }
                Command::RemoveEdges(n) => {
                    for _ in 0..n {
                        self.remove_random_edge();
                    }
                }
                Command::SwapEdges(n) => {
                    for _ in 0..n {
                        self.remove_random_edge();
                        self.add_random_edge();
                    }
                }
            }
        }

        // Detect any key/pointer press this frame to support "close on any interaction"
        let mut any_key_pressed = false;
        let mut any_pointer_pressed = false;
        ctx.input(|i| {
            for ev in &i.events {
                match ev {
                    egui::Event::Key { pressed, .. } => {
                        if *pressed {
                            any_key_pressed = true;
                        }
                    }
                    egui::Event::PointerButton { pressed, .. } => {
                        if *pressed {
                            any_pointer_pressed = true;
                        }
                    }
                    _ => {}
                }
            }
        });

        if open_modal {
            self.show_keybindings_overlay = true;
            self.keybindings_just_opened = true;
        }
        if self.show_keybindings_overlay {
            if !self.keybindings_just_opened && (any_key_pressed || any_pointer_pressed) {
                close_modal = true;
            }
            // Clear the guard at end of frame
            self.keybindings_just_opened = false;
        }
        if close_modal {
            self.show_keybindings_overlay = false;
        }
    }
}
