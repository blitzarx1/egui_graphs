use core::cmp::Ordering;
use eframe::{App, CreationContext};
use egui::{self, CollapsingHeader, Pos2, ScrollArea, Ui};
use egui_graphs::{
    generate_random_graph, FruchtermanReingoldWithCenterGravity,
    FruchtermanReingoldWithCenterGravityState, Graph, LayoutForceDirected,
};
use petgraph::stable_graph::{DefaultIx, EdgeIndex, NodeIndex};
use petgraph::Directed;
use rand::Rng;
#[cfg(not(feature = "events"))]
use std::time::Duration;
use instant::Instant;

pub const MAX_NODE_COUNT: usize = 2500;
pub const MAX_EDGE_COUNT: usize = 5000;
#[cfg(feature = "events")]
pub const EVENTS_LIMIT: usize = 200;

#[cfg(feature = "events")]
pub use crossbeam::channel::{unbounded, Receiver, Sender};
#[cfg(feature = "events")]
pub use egui_graphs::events::Event;

pub mod settings_local {
    pub struct SettingsInteraction {
        pub dragging_enabled: bool,
        pub hover_enabled: bool,
        pub node_clicking_enabled: bool,
        pub node_selection_enabled: bool,
        pub node_selection_multi_enabled: bool,
        pub edge_clicking_enabled: bool,
        pub edge_selection_enabled: bool,
        pub edge_selection_multi_enabled: bool,
    }
    impl Default for SettingsInteraction {
        fn default() -> Self {
            Self {
                dragging_enabled: true,
                hover_enabled: true,
                node_clicking_enabled: false,
                node_selection_enabled: false,
                node_selection_multi_enabled: false,
                edge_clicking_enabled: false,
                edge_selection_enabled: false,
                edge_selection_multi_enabled: false,
            }
        }
    }

    #[derive(Default)]
    pub struct SettingsStyle {
        pub labels_always: bool,
        pub edge_deemphasis: bool,
    }

    pub struct SettingsNavigation {
        pub fit_to_screen_enabled: bool,
        pub zoom_and_pan_enabled: bool,
        pub zoom_speed: f32,
        pub fit_to_screen_padding: f32,
    }
    impl Default for SettingsNavigation {
        fn default() -> Self {
            Self {
                fit_to_screen_enabled: true,
                zoom_and_pan_enabled: false,
                zoom_speed: 0.1,
                fit_to_screen_padding: 0.1,
            }
        }
    }

    pub struct SettingsGraph {
        pub count_node: usize,
        pub count_edge: usize,
    }
    impl Default for SettingsGraph {
        fn default() -> Self {
            Self { count_node: 25, count_edge: 50 }
        }
    }
}
pub use settings_local as settings;

fn info_icon(ui: &mut egui::Ui, tip: &str) {
    ui.add_space(4.0);
    ui.small_button("ℹ").on_hover_text(tip);
}

pub mod drawers {
    use crate::{MAX_EDGE_COUNT, MAX_NODE_COUNT};
    use egui::Ui;

    pub struct GraphCountSliders {
        pub nodes: usize,
        pub edges: usize,
    }

    pub fn graph_count_sliders(
        ui: &mut Ui,
        mut v: GraphCountSliders,
        mut on_change: impl FnMut(i32, i32),
    ) {
        let mut delta_nodes: i32 = 0;
        let mut delta_edges: i32 = 0;

        ui.horizontal(|ui| {
            let start = v.nodes;
            ui.label("N");
            ui.add(egui::Slider::new(&mut v.nodes, 0..=MAX_NODE_COUNT));
            if ui.small_button("-10").on_hover_text("Remove 10 nodes (M)").clicked() {
                v.nodes = v.nodes.saturating_sub(10);
            }
            if ui.small_button("-1").on_hover_text("Remove 1 node (N)").clicked() {
                v.nodes = v.nodes.saturating_sub(1);
            }
            if ui.small_button("+1").on_hover_text("Add 1 node (n)").clicked() {
                v.nodes = (v.nodes + 1).min(MAX_NODE_COUNT);
            }
            if ui.small_button("+10").on_hover_text("Add 10 nodes (m)").clicked() {
                v.nodes = (v.nodes + 10).min(MAX_NODE_COUNT);
            }
            delta_nodes = if v.nodes >= start {
                i32::try_from(v.nodes - start).unwrap()
            } else {
                -i32::try_from(start - v.nodes).unwrap()
            };
        });

        ui.horizontal(|ui| {
            let start = v.edges;
            ui.label("E");
            ui.add(egui::Slider::new(&mut v.edges, 0..=MAX_EDGE_COUNT));
            if ui.small_button("-10").on_hover_text("Remove 10 edges (R)").clicked() {
                v.edges = v.edges.saturating_sub(10);
            }
            if ui.small_button("-1").on_hover_text("Remove 1 edge (E)").clicked() {
                v.edges = v.edges.saturating_sub(1);
            }
            if ui.small_button("+1").on_hover_text("Add 1 edge (e)").clicked() {
                v.edges = (v.edges + 1).min(MAX_EDGE_COUNT);
            }
            if ui.small_button("+10").on_hover_text("Add 10 edges (r)").clicked() {
                v.edges = (v.edges + 10).min(MAX_EDGE_COUNT);
            }
            delta_edges = if v.edges >= start {
                i32::try_from(v.edges - start).unwrap()
            } else {
                -i32::try_from(start - v.edges).unwrap()
            };
        });

        if delta_nodes != 0 || delta_edges != 0 {
            on_change(delta_nodes, delta_edges);
        }
    }
}

pub struct DemoApp {
    pub g: Graph<(), (), Directed, DefaultIx>,
    pub settings_graph: settings::SettingsGraph,
    pub settings_interaction: settings::SettingsInteraction,
    pub settings_navigation: settings::SettingsNavigation,
    pub settings_style: settings::SettingsStyle,
    pub fps: f32,
    pub last_update_time: Instant,
    pub frames_last_time_span: usize,
    pub show_sidebar: bool,
    pub dark_mode: bool,
    pub show_debug_overlay: bool,
    pub show_keybindings_overlay: bool,
    pub keybindings_just_opened: bool,
    pub reset_requested: bool,
    pub last_step_count: usize,
    #[cfg(not(feature = "events"))]
    pub copy_tip_until: Option<Instant>,
    #[cfg(feature = "events")]
    pub pan: [f32; 2],
    #[cfg(feature = "events")]
    pub zoom: f32,
    #[cfg(feature = "events")]
    pub last_events: Vec<String>,
    #[cfg(feature = "events")]
    pub event_publisher: crate::Sender<Event>,
    #[cfg(feature = "events")]
    pub event_consumer: crate::Receiver<Event>,
    #[cfg(feature = "events")]
    pub event_filters: EventFilters,
}
impl DemoApp {
    pub fn new(cc: &CreationContext<'_>) -> Self {
        let settings_graph = settings::SettingsGraph::default();
        let mut g = generate_random_graph(settings_graph.count_node, settings_graph.count_edge);
        Self::distribute_nodes_circle(&mut g);

        #[cfg(feature = "events")]
        let (event_publisher, event_consumer) = crate::unbounded();

        Self {
            g,
            settings_graph,
            settings_interaction: settings::SettingsInteraction::default(),
            settings_navigation: settings::SettingsNavigation::default(),
            settings_style: settings::SettingsStyle { labels_always: false, edge_deemphasis: true },
            fps: 0.0,
            last_update_time: Instant::now(),
            frames_last_time_span: 0,
            show_sidebar: true,
            #[cfg(not(feature = "events"))]
            copy_tip_until: None,
            #[cfg(feature = "events")]
            pan: [0.0, 0.0],
            #[cfg(feature = "events")]
            zoom: 1.0,
            #[cfg(feature = "events")]
            last_events: Vec::new(),
            #[cfg(feature = "events")]
            event_publisher,
            #[cfg(feature = "events")]
            event_consumer,
            #[cfg(feature = "events")]
            event_filters: EventFilters::default(),
            dark_mode: cc.egui_ctx.style().visuals.dark_mode,
            show_debug_overlay: true,
            show_keybindings_overlay: false,
            keybindings_just_opened: false,
            reset_requested: false,
            last_step_count: 0,
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
        if self.g.node_count() >= MAX_NODE_COUNT { return; }
        if let Some(r) = self.random_node_idx() {
            let n = self.g.node(r).unwrap();
            let mut rng = rand::rng();
            let loc = Pos2::new(
                n.location().x + 10.0 + rng.random_range(0.0..50.0),
                n.location().y + 10.0 + rng.random_range(0.0..50.0),
            );
            self.g.add_node_with_location((), loc);
        } else {
            self.g.add_node(());
        }
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
        if self.g.edge_count() >= MAX_EDGE_COUNT { return; }
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
        self.frames_last_time_span += 1;
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_update_time);
        if elapsed.as_secs() >= 1 {
            self.last_update_time = now;
            self.fps = self.frames_last_time_span as f32 / elapsed.as_secs_f32();
            self.frames_last_time_span = 0;
        }
    }

    pub fn ui_graph_section(&mut self, ui: &mut Ui) {
        crate::drawers::graph_count_sliders(
            ui,
            crate::drawers::GraphCountSliders { nodes: self.settings_graph.count_node, edges: self.settings_graph.count_edge },
            |dn, de| {
                match dn.cmp(&0) {
                    Ordering::Greater => { for _ in 0..dn { self.add_random_node(); } },
                    Ordering::Less => { for _ in 0..(-dn) { self.remove_random_node(); } },
                    Ordering::Equal => {}
                }
                match de.cmp(&0) {
                    Ordering::Greater => { for _ in 0..de { self.add_random_edge(); } },
                    Ordering::Less => { for _ in 0..(-de) { self.remove_random_edge(); } },
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
        self.settings_style = settings::SettingsStyle { labels_always: false, edge_deemphasis: true };
        self.show_debug_overlay = true;
        self.show_keybindings_overlay = false;
        self.g = generate_random_graph(self.settings_graph.count_node, self.settings_graph.count_edge);
        Self::distribute_nodes_circle(&mut self.g);
        egui_graphs::GraphView::<
            (), (), petgraph::Directed, petgraph::stable_graph::DefaultIx,
            egui_graphs::DefaultNodeShape, egui_graphs::DefaultEdgeShape,
            FruchtermanReingoldWithCenterGravityState,
            LayoutForceDirected<FruchtermanReingoldWithCenterGravity>,
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
        self.fps = 0.0;
    }

    pub fn distribute_nodes_circle(g: &mut Graph<(), (), Directed, DefaultIx>) {
        let n_usize = core::cmp::max(g.node_count(), 1);
        if n_usize == 0 { return; }
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

    pub fn ui_layout_force_directed(&mut self, ui: &mut Ui) {
        let mut state = egui_graphs::GraphView::<
            (), (), petgraph::Directed, petgraph::stable_graph::DefaultIx,
            egui_graphs::DefaultNodeShape, egui_graphs::DefaultEdgeShape,
            FruchtermanReingoldWithCenterGravityState,
            LayoutForceDirected<FruchtermanReingoldWithCenterGravity>,
        >::get_layout_state(ui);

        CollapsingHeader::new("Force Directed Layout").default_open(true).show(ui, |ui| {
            fn info_icon(ui: &mut egui::Ui, tip: &str) {
                ui.add_space(4.0);
                if ui.small_button("ℹ").on_hover_text(tip).clicked() {}
            }

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
        });

        egui_graphs::GraphView::<
            (), (), petgraph::Directed, petgraph::stable_graph::DefaultIx,
            egui_graphs::DefaultNodeShape, egui_graphs::DefaultEdgeShape,
            FruchtermanReingoldWithCenterGravityState,
            LayoutForceDirected<FruchtermanReingoldWithCenterGravity>,
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
                if ui.checkbox(&mut dark, "dark mode").on_hover_text("Toggle dark or light visuals").changed() {
                    if dark { ui.ctx().set_visuals(egui::Visuals::dark()); } else { ui.ctx().set_visuals(egui::Visuals::light()); }
                    self.dark_mode = dark;
                } else {
                    self.dark_mode = dark;
                }
                info_icon(ui, "Synced with global egui style context for seamless integration.");
            });
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.checkbox(&mut self.settings_style.labels_always, "labels_always");
                info_icon(ui, "Always render node & edge labels instead of only on interaction.");
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
        CollapsingHeader::new("Selected").default_open(true).show(ui, |ui| {
            ScrollArea::vertical().max_height(150.0).show(ui, |ui| {
                for n in self.g.selected_nodes() { ui.label(format!("{n:?}")); }
                for e in self.g.selected_edges() { ui.label(format!("{e:?}")); }
            });
        });
    }

    #[cfg(not(feature = "events"))]
    pub fn show_events_feature_tip(&mut self, ui: &mut Ui) {
        ui.group(|ui| {
            ui.colored_label(egui::Color32::from_rgb(200, 180, 40),
                "Tip: enable the 'events' feature to see interaction events (pan/zoom, clicks, selections).",
            );

            let cmd = "cargo r --release --example demo --features events";
            ui.horizontal(|ui| {
                ui.code(cmd);
                if ui.small_button("📋").on_hover_text("Copy command").clicked() {
                    ui.ctx().copy_text(cmd.to_string());
                    self.copy_tip_until = Some(Instant::now() + Duration::from_millis(1500));
                    ui.ctx().request_repaint();
                }
            });
            if let Some(until) = self.copy_tip_until {
                if Instant::now() < until {
                    ui.label(egui::RichText::new("Copied!").color(egui::Color32::LIGHT_GREEN).small());
                    ui.ctx().request_repaint_after(Duration::from_millis(100));
                } else {
                    self.copy_tip_until = None;
                }
            }
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
            egui::SidePanel::right("right").default_width(300.0).min_width(300.0).show(ctx, |ui| {
                ScrollArea::vertical().show(ui, |ui| {
                    #[cfg(not(feature = "events"))]
                    self.show_events_feature_tip(ui);
                    if ui.button("Reset Defaults").on_hover_text("Reset ALL settings, graph, layout & view state (Space)").clicked() {
                        self.reset_all(ui);
                    }
                    CollapsingHeader::new("Graph / Layout").default_open(true).show(ui, |ui| self.ui_graph_section(ui));
                    self.ui_navigation(ui);
                    self.ui_layout_force_directed(ui);
                    self.ui_interaction(ui);
                    self.ui_style(ui);
                    self.ui_debug(ui);
                    self.ui_selected(ui);
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
                .with_node_selection_multi_enabled(self.settings_interaction.node_selection_multi_enabled)
                .with_dragging_enabled(self.settings_interaction.dragging_enabled)
                .with_hover_enabled(self.settings_interaction.hover_enabled)
                .with_node_clicking_enabled(self.settings_interaction.node_clicking_enabled)
                .with_edge_clicking_enabled(self.settings_interaction.edge_clicking_enabled)
                .with_edge_selection_enabled(self.settings_interaction.edge_selection_enabled)
                .with_edge_selection_multi_enabled(self.settings_interaction.edge_selection_multi_enabled);
            let settings_navigation = &egui_graphs::SettingsNavigation::new()
                .with_zoom_and_pan_enabled(self.settings_navigation.zoom_and_pan_enabled)
                .with_fit_to_screen_enabled(self.settings_navigation.fit_to_screen_enabled)
                .with_zoom_speed(self.settings_navigation.zoom_speed)
                .with_fit_to_screen_padding(self.settings_navigation.fit_to_screen_padding);
            let mut style_builder = egui_graphs::SettingsStyle::new().with_labels_always(self.settings_style.labels_always);
            if self.settings_style.edge_deemphasis {
                style_builder = style_builder.with_edge_stroke_hook(|selected, _order, stroke, _style| {
                    let mut s = stroke;
                    if !selected {
                        let c = s.color;
                        let new_a = (f32::from(c.a()) * 0.5) as u8;
                        s.color = egui::Color32::from_rgba_unmultiplied(c.r(), c.g(), c.b(), new_a);
                    }
                    s
                });
            }
            let settings_style = &style_builder;

            let mut view = egui_graphs::GraphView::<
                _, _, _, _, _, _,
                FruchtermanReingoldWithCenterGravityState,
                LayoutForceDirected<FruchtermanReingoldWithCenterGravity>,
            >::new(&mut self.g)
                .with_interactions(settings_interaction)
                .with_navigations(settings_navigation)
                .with_styles(settings_style);
            #[cfg(feature = "events")]
            { view = view.with_events(&self.event_publisher); }
            ui.add(&mut view);

            // Capture latest layout step count for overlay display
            let state = egui_graphs::GraphView::<
                (), (), petgraph::Directed, petgraph::stable_graph::DefaultIx,
                egui_graphs::DefaultNodeShape, egui_graphs::DefaultEdgeShape,
                FruchtermanReingoldWithCenterGravityState,
                LayoutForceDirected<FruchtermanReingoldWithCenterGravity>,
            >::get_layout_state(ui);
            self.last_step_count = state.base.step_count as usize;

            // Draw overlay inside the CentralPanel so it stays within the graph area
            self.overlay_debug_panel(ui);
            // Draw toggle button for the side panel at bottom-right of the graph area
            self.overlay_toggle_sidebar_button(ui);
        });

        self.update_fps();

    // Draw modal after main UI
    self.keybindings_modal(ctx);
    }
}

impl DemoApp {
    fn overlay_debug_panel(&mut self, ui: &mut egui::Ui) {
        if !self.show_debug_overlay {
            return;
        }

        // Compose overlay text
        let text = {
            let fps_line = format!("FPS: {:.1}", self.fps);
            let node_count = self.g.node_count();
            let edge_count = self.g.edge_count();
            let n_line = if node_count >= MAX_NODE_COUNT { format!("N: {node_count} MAX") } else { format!("N: {node_count}") };
            let e_line = if edge_count >= MAX_EDGE_COUNT { format!("E: {edge_count} MAX") } else { format!("E: {edge_count}") };
            let steps_line = format!("Steps: {}", self.last_step_count);
            #[cfg(feature = "events")]
            let zoom_line = format!("Zoom: {:.3}", self.zoom);
            #[cfg(feature = "events")]
            let pan_line = format!("Pan: [{:.1},{:.1}]", self.pan[0], self.pan[1]);

            #[cfg(feature = "events")]
            { format!("{fps_line}\n{n_line}\n{e_line}\n{steps_line}\n{zoom_line}\n{pan_line}") }
            #[cfg(not(feature = "events"))]
            { format!(
                "{fps_line}\n{n_line}\n{e_line}\n{steps_line}\nZoom: enable events feature\nPan: enable events feature"
            ) }
        };

    // Measure text using monospace font
    let font_id = egui::FontId::monospace(14.0);
    let text_color = ui.style().visuals.strong_text_color();
    let text_size = ui.fonts(|f| f.layout_no_wrap(text.clone(), font_id.clone(), text_color).size());

    // Position (no frame) based on text size
    let margin = 10.0;
    let panel_rect = ui.max_rect();
    let pos = egui::pos2(panel_rect.max.x - margin - text_size.x, panel_rect.min.y + margin);

    // Draw overlay text only (no frame) within the CentralPanel rect.
        egui::Area::new(egui::Id::new("overlay_debug_in_panel"))
            .order(egui::Order::Middle)
            .fixed_pos(pos)
            .movable(false)
            .show(ui.ctx(), |ui_area| {
                // clip to panel to ensure it doesn't draw outside
                ui_area.set_clip_rect(panel_rect);
        ui_area.style_mut().override_text_style = Some(egui::TextStyle::Monospace);
        ui_area.label(
            egui::RichText::new(text)
            .monospace()
            .color(text_color)
            .size(14.0),
        );
            });
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
        // Small overlay button inside the CentralPanel to toggle the right side panel
        let g_rect = ui.max_rect();
        let btn_size = egui::vec2(28.0, 28.0);
        // Use the same external padding as the debug overlay (10px)
        let right_margin = 10.0;
        let bottom_margin = 10.0;
        let btn_pos = egui::pos2(
            g_rect.right() - right_margin - btn_size.x,
            g_rect.bottom() - bottom_margin - btn_size.y,
        );

        // Use filled triangles for a nicer look
        let (arrow, tip) = if self.show_sidebar {
            ("▶", "Hide side panel (Tab)")
        } else {
            ("◀", "Show side panel (Tab)")
        };

        egui::Area::new(egui::Id::new("sidebar_toggle_btn"))
            .order(egui::Order::Middle)
            .fixed_pos(btn_pos)
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
    fn process_keybindings(&mut self, ctx: &egui::Context) {
        // Toggle modal on 'h' or '?' and close on any interaction after open.
        let mut any_key_pressed = false;
        let mut any_pointer_pressed = false;
        let mut pressed_h = false;            // 'h' or 'H'
        let mut pressed_shift_slash = false; // '?'
        ctx.input(|i| {
            for ev in &i.events {
                match ev {
                    egui::Event::Key { key, pressed, modifiers, .. } => {
                        if *pressed { any_key_pressed = true; }
                        if *pressed && !modifiers.any() && *key == egui::Key::H { pressed_h = true; }
                        if *pressed && *key == egui::Key::Slash && modifiers.shift { pressed_shift_slash = true; }
                    }
                    egui::Event::PointerButton { pressed, .. } => { if *pressed { any_pointer_pressed = true; } }
                    egui::Event::Text(t) => {
                        if t == "?" { pressed_shift_slash = true; }
                        if t.eq_ignore_ascii_case("h") { pressed_h = true; }
                    }
                    _ => {}
                }
            }
        });
        let mut open_modal = false;
        let mut close_modal = false;
        if pressed_h || pressed_shift_slash {
            if self.show_keybindings_overlay { close_modal = true; } else { open_modal = true; }
        }
        if open_modal {
            self.show_keybindings_overlay = true;
            self.keybindings_just_opened = true;
        }
        if self.show_keybindings_overlay {
            if !self.keybindings_just_opened && (any_key_pressed || any_pointer_pressed) {
                close_modal = true;
            }
            // Clear the "just opened" guard at the end of the frame
            self.keybindings_just_opened = false;
        }
        if close_modal { self.show_keybindings_overlay = false; }

        // Other shortcuts
        ctx.input(|i| {
            // Space: reset defaults
            if i.key_pressed(egui::Key::Space) && !i.modifiers.any() { self.reset_requested = true; }
            // Esc: close modal if open
            if i.key_pressed(egui::Key::Escape) {
                self.show_keybindings_overlay = false;
            }
            for ev in &i.events {
                if let egui::Event::Key { key, pressed, modifiers, .. } = ev {
                    if !pressed { continue; }
                    match key {
                        egui::Key::Tab => {
                            if !modifiers.any() { self.show_sidebar = !self.show_sidebar; }
                        }
                        egui::Key::D => { if !modifiers.any() { self.show_debug_overlay = !self.show_debug_overlay; } }
                        egui::Key::N => {
                            if modifiers.ctrl && modifiers.shift { self.remove_random_node(); self.add_random_node(); }
                            else if modifiers.shift { self.remove_random_node(); }
                            else { self.add_random_node(); }
                        }
                        egui::Key::M => {
                            if modifiers.ctrl && modifiers.shift { for _ in 0..10 { self.remove_random_node(); } for _ in 0..10 { self.add_random_node(); } }
                            else if modifiers.shift { for _ in 0..10 { self.remove_random_node(); } }
                            else {
                                let remaining = MAX_NODE_COUNT.saturating_sub(self.g.node_count());
                                let to_add = remaining.min(10);
                                for _ in 0..to_add { self.add_random_node(); }
                            }
                        }
                        egui::Key::E => {
                            if modifiers.ctrl && modifiers.shift { self.remove_random_edge(); self.add_random_edge(); }
                            else if modifiers.shift { self.remove_random_edge(); }
                            else { self.add_random_edge(); }
                        }
                        egui::Key::R => {
                            if modifiers.ctrl && modifiers.shift { for _ in 0..10 { self.remove_random_edge(); } for _ in 0..10 { self.add_random_edge(); } }
                            else if modifiers.shift { for _ in 0..10 { self.remove_random_edge(); } }
                            else {
                                let remaining = MAX_EDGE_COUNT.saturating_sub(self.g.edge_count());
                                let to_add = remaining.min(10);
                                for _ in 0..to_add { self.add_random_edge(); }
                            }
                        }
                        _ => {}
                    }
                }
            }
        });
    }
}

#[cfg(feature = "events")]
#[derive(Clone)]
pub struct EventFilters {
    pub pan: bool,
    pub zoom: bool,
    pub node_move: bool,
    pub node_drag_start: bool,
    pub node_drag_end: bool,
    pub node_hover_enter: bool,
    pub node_hover_leave: bool,
    pub node_select: bool,
    pub node_deselect: bool,
    pub node_click: bool,
    pub node_double_click: bool,
    pub edge_click: bool,
    pub edge_select: bool,
    pub edge_deselect: bool,
}

#[cfg(feature = "events")]
impl Default for EventFilters {
    fn default() -> Self {
        Self { pan: true, zoom: true, node_move: true, node_drag_start: true, node_drag_end: true, node_hover_enter: true, node_hover_leave: true, node_select: true, node_deselect: true, node_click: true, node_double_click: true, edge_click: true, edge_select: true, edge_deselect: true }
    }
}

#[cfg(feature = "events")]
impl EventFilters {
    pub fn enabled_for(&self, e: &Event) -> bool {
        use Event::*;
        match e {
            Pan(_) => self.pan,
            Zoom(_) => self.zoom,
            NodeMove(_) => self.node_move,
            NodeDragStart(_) => self.node_drag_start,
            NodeDragEnd(_) => self.node_drag_end,
            NodeHoverEnter(_) => self.node_hover_enter,
            NodeHoverLeave(_) => self.node_hover_leave,
            NodeSelect(_) => self.node_select,
            NodeDeselect(_) => self.node_deselect,
            NodeClick(_) => self.node_click,
            NodeDoubleClick(_) => self.node_double_click,
            EdgeClick(_) => self.edge_click,
            EdgeSelect(_) => self.edge_select,
            EdgeDeselect(_) => self.edge_deselect,
        }
    }
    pub fn is_event_str_enabled(&self, ev: &str) -> Option<bool> {
        if ev.starts_with("Pan") { Some(self.pan) }
        else if ev.starts_with("Zoom") { Some(self.zoom) }
        else if ev.starts_with("NodeMove") { Some(self.node_move) }
        else if ev.starts_with("NodeDragStart") { Some(self.node_drag_start) }
        else if ev.starts_with("NodeDragEnd") { Some(self.node_drag_end) }
        else if ev.starts_with("NodeHoverEnter") { Some(self.node_hover_enter) }
        else if ev.starts_with("NodeHoverLeave") { Some(self.node_hover_leave) }
        else if ev.starts_with("NodeSelect") { Some(self.node_select) }
        else if ev.starts_with("NodeDeselect") { Some(self.node_deselect) }
        else if ev.starts_with("NodeClick") { Some(self.node_click) }
        else if ev.starts_with("NodeDoubleClick") { Some(self.node_double_click) }
        else if ev.starts_with("EdgeClick") { Some(self.edge_click) }
        else if ev.starts_with("EdgeSelect") { Some(self.edge_select) }
        else if ev.starts_with("EdgeDeselect") { Some(self.edge_deselect) }
        else { None }
    }
    pub fn purge_disabled(&self, events: &mut Vec<String>) {
        events.retain(|ev| match self.is_event_str_enabled(ev.as_str()) { Some(enabled) => enabled, None => true });
    }
}
