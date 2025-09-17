use core::cmp::Ordering;
use eframe::{App, CreationContext};
use egui::{self, Align2, CollapsingHeader, Color32, Pos2, Rect, ScrollArea, Ui};
use egui_graphs::{
    generate_random_graph, FruchtermanReingoldWithCenterGravity,
    FruchtermanReingoldWithCenterGravityState, Graph, LayoutForceDirected, LayoutHierarchical,
    LayoutHierarchicalOrientation, LayoutStateHierarchical,
};
#[cfg(not(feature = "events"))]
use instant::Instant;
use petgraph::stable_graph::{DefaultIx, EdgeIndex, NodeIndex};
use petgraph::{Directed, Undirected};
use rand::Rng;
#[cfg(all(feature = "events", target_arch = "wasm32"))]
use std::{cell::RefCell, rc::Rc};

mod event_filters;
mod graph_ops;
mod import;
mod keybindings;
mod metrics;
mod overlays;
mod spec;
mod status;
mod tabs;
#[cfg(all(target_arch = "wasm32", not(feature = "events")))]
use std::{cell::RefCell, rc::Rc};
use tabs::import_load::UserUpload;
mod ui_consts;
mod util;

pub const MAX_NODE_COUNT: usize = 2500;
pub const MAX_EDGE_COUNT: usize = 5000;
#[cfg(feature = "events")]
pub const EVENTS_LIMIT: usize = 500;
// Keep margins consistent for overlays/buttons in the CentralPanel
use ui_consts::{
    HEADING_TEXT_SIZE, OVERLAY_BTN_SIZE, OVERLAY_BTN_SPACING, OVERLAY_ICON_SIZE, SECTION_SPACING,
    SELECTED_SCROLL_MAX_HEIGHT, SIDE_PANEL_WIDTH, UI_MARGIN,
};

#[cfg(feature = "events")]
use crate::event_filters::EventFilters;
use crate::graph_ops::GraphActions;
use crate::keybindings::{dispatch as dispatch_keybindings, Command};
use crate::metrics::MetricsRecorder;
use crate::status::{StatusKind, StatusQueue};
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

#[derive(Debug)]
pub enum DemoGraph {
    Directed(Graph<(), (), Directed, DefaultIx>),
    Undirected(Graph<(), (), Undirected, DefaultIx>),
}

// Perf metrics routing for Directed vs Undirected and selected layout
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MetricsRoute {
    DirectedFR,
    UndirectedFR,
    DirectedHier,
    UndirectedHier,
}

fn pick_metrics_route(g: &DemoGraph, layout: DemoLayout) -> MetricsRoute {
    match (layout, g) {
        (DemoLayout::FruchtermanReingold, DemoGraph::Directed(_)) => MetricsRoute::DirectedFR,
        (DemoLayout::FruchtermanReingold, DemoGraph::Undirected(_)) => MetricsRoute::UndirectedFR,
        (DemoLayout::Hierarchical, DemoGraph::Directed(_)) => MetricsRoute::DirectedHier,
        (DemoLayout::Hierarchical, DemoGraph::Undirected(_)) => MetricsRoute::UndirectedHier,
    }
}

// Main demo application state
pub struct DemoApp {
    pub g: DemoGraph,
    pub settings_graph: settings::SettingsGraph,
    pub settings_interaction: settings::SettingsInteraction,
    pub settings_navigation: settings::SettingsNavigation,
    pub settings_style: settings::SettingsStyle,
    pub metrics: MetricsRecorder,
    // UI
    pub show_sidebar: bool,
    #[cfg(not(feature = "events"))]
    pub copy_tip_until: Option<Instant>,
    // Events (feature gated)
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
    // Misc
    pub dark_mode: bool,
    pub show_debug_overlay: bool,
    pub show_keybindings_overlay: bool,
    pub keybindings_just_opened: bool,
    pub reset_requested: bool,
    pub drag_hover_graph: bool,
    pub status: StatusQueue,
    pub selected_layout: DemoLayout,
    // Whether any text input is focused this frame (to disable global Tab handling)
    pub typing_in_input: bool,
    // Export modal state
    pub show_export_modal: bool,
    pub export_include_layout: bool,
    pub export_include_graph: bool,
    pub export_include_positions: bool,
    pub export_destination: ExportDestination,
    pub export_filename: String,
    // If an import provided a layout state, apply it on next UI frame
    pub pending_layout: Option<spec::PendingLayout>,
    // Right panel tabs
    pub right_tab: RightTab,
    // Saved user uploads (JSON text)
    pub user_uploads: Vec<UserUpload>,
    #[cfg(target_arch = "wasm32")]
    pub web_upload_buf: Rc<RefCell<Vec<UserUpload>>>,
    // One-shot navigation actions
    pub fit_to_screen_once_pending: bool,
    pub pan_to_graph_pending: bool,
}

// (removed malformed early impl DemoApp)

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DemoLayout {
    FruchtermanReingold,
    Hierarchical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportDestination {
    File,
    Clipboard,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RightTab {
    Playground,
    Import,
}

impl DemoApp {
    pub fn new(cc: &CreationContext<'_>) -> Self {
        let settings_graph = settings::SettingsGraph::default();
        let mut g = generate_random_graph(settings_graph.count_node, settings_graph.count_edge);
        Self::distribute_nodes_circle_generic(&mut g);

        #[cfg(all(feature = "events", not(target_arch = "wasm32")))]
        let (event_publisher, event_consumer) = crate::unbounded();
        #[cfg(all(feature = "events", target_arch = "wasm32"))]
        let events_buf: Rc<RefCell<Vec<Event>>> = Rc::new(RefCell::new(Vec::new()));

        #[allow(unused_mut)]
        let mut app = Self {
            g: DemoGraph::Directed(g),
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
            drag_hover_graph: false,
            status: StatusQueue::new(),
            selected_layout: DemoLayout::FruchtermanReingold,
            typing_in_input: false,
            show_export_modal: false,
            export_include_layout: true,
            export_include_graph: true,
            export_include_positions: false,
            export_destination: ExportDestination::File,
            export_filename: crate::util::default_export_filename(),
            pending_layout: None,
            right_tab: RightTab::Playground,
            user_uploads: Vec::new(),
            #[cfg(target_arch = "wasm32")]
            web_upload_buf: Rc::new(RefCell::new(Vec::new())),
            fit_to_screen_once_pending: false,
            pan_to_graph_pending: false,
        };

        // Web: if URL hash contains g=<example_name>, load that example graph automatically
        #[cfg(target_arch = "wasm32")]
        {
            if let Some(name) = crate::web_hash_get_param("g") {
                if let Some(data) = crate::web_lookup_example_asset(&name) {
                    app.load_graph_from_str(&name, data);
                }
            }
        }

        app
    }

    pub fn random_node_idx(&self) -> Option<NodeIndex> {
        // Moved into GraphActions; keep thin wrapper if still referenced elsewhere.
        let cnt = match &self.g {
            DemoGraph::Directed(g) => g.node_count(),
            DemoGraph::Undirected(g) => g.node_count(),
        };
        if cnt == 0 {
            return None;
        }
        let idx = rand::rng().random_range(0..cnt);
        match &self.g {
            DemoGraph::Directed(g) => g.g().node_indices().nth(idx),
            DemoGraph::Undirected(g) => g.g().node_indices().nth(idx),
        }
    }
    pub fn random_edge_idx(&self) -> Option<EdgeIndex> {
        // Moved into GraphActions; keep thin wrapper if still referenced elsewhere.
        let cnt = match &self.g {
            DemoGraph::Directed(g) => g.edge_count(),
            DemoGraph::Undirected(g) => g.edge_count(),
        };
        if cnt == 0 {
            return None;
        }
        let idx = rand::rng().random_range(0..cnt);
        match &self.g {
            DemoGraph::Directed(g) => g.g().edge_indices().nth(idx),
            DemoGraph::Undirected(g) => g.g().edge_indices().nth(idx),
        }
    }
    pub fn add_random_node(&mut self) {
        GraphActions { g: &mut self.g }.add_random_node();
    }
    pub fn remove_random_node(&mut self) {
        GraphActions { g: &mut self.g }.remove_random_node();
    }
    pub fn remove_node(&mut self, idx: NodeIndex) {
        GraphActions { g: &mut self.g }.remove_node_by_idx(idx);
    }
    pub fn add_random_edge(&mut self) {
        GraphActions { g: &mut self.g }.add_random_edge();
    }
    pub fn add_edge(&mut self, a: NodeIndex, b: NodeIndex) {
        GraphActions { g: &mut self.g }.add_edge(a, b);
    }
    pub fn remove_random_edge(&mut self) {
        GraphActions { g: &mut self.g }.remove_random_edge();
    }
    pub fn remove_edge(&mut self, a: NodeIndex, b: NodeIndex) {
        GraphActions { g: &mut self.g }.remove_edge(a, b);
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
                let mut ga = GraphActions { g: &mut self.g };
                match dn.cmp(&0) {
                    Ordering::Greater => {
                        ga.add_nodes(dn as u32);
                    }
                    Ordering::Less => {
                        ga.remove_nodes((-dn) as u32);
                    }
                    Ordering::Equal => {}
                }
                match de.cmp(&0) {
                    Ordering::Greater => {
                        ga.add_edges(de as u32);
                    }
                    Ordering::Less => {
                        ga.remove_edges((-de) as u32);
                    }
                    Ordering::Equal => {}
                }
                let (n, e) = match &self.g {
                    DemoGraph::Directed(g) => (g.node_count(), g.edge_count()),
                    DemoGraph::Undirected(g) => (g.node_count(), g.edge_count()),
                };
                self.settings_graph.count_node = n;
                self.settings_graph.count_edge = e;
            },
        );

        ui.separator();
        // Toggle graph directedness
        let mut directed = matches!(self.g, DemoGraph::Directed(_));
        ui.horizontal(|ui| {
            if ui.checkbox(&mut directed, "Directed").clicked() {
                self.set_directedness(directed);
            }
            info_icon(ui, "Toggle directed edges (arrowheads, ordering, layouts/metrics). Positions are preserved when switching.");
        });
    }

    fn set_directedness(&mut self, directed: bool) {
        use std::collections::{HashMap, HashSet};
        match (&self.g, directed) {
            (DemoGraph::Directed(_), true) | (DemoGraph::Undirected(_), false) => {}
            (DemoGraph::Directed(gd), false) => {
                // Directed -> Undirected (dedupe unordered pairs)
                let src = gd.clone();
                let mut sg: petgraph::stable_graph::StableGraph<
                    (),
                    (),
                    petgraph::Undirected,
                    DefaultIx,
                > = Default::default();
                let mut map: HashMap<NodeIndex, NodeIndex> = HashMap::new();
                for ni in src.g().node_indices() {
                    let new_ni = sg.add_node(());
                    map.insert(ni, new_ni);
                }
                let mut seen: HashSet<(usize, usize)> = HashSet::new();
                for ei in src.g().edge_indices() {
                    if let Some((a, b)) = src.g().edge_endpoints(ei) {
                        let (u, v) = if a.index() <= b.index() {
                            (a, b)
                        } else {
                            (b, a)
                        };
                        if seen.insert((u.index(), v.index())) {
                            let ai = map[&u];
                            let bi = map[&v];
                            let _ = sg.add_edge(ai, bi, ());
                        }
                    }
                }
                let mut dst: egui_graphs::Graph<(), (), petgraph::Undirected, DefaultIx> =
                    egui_graphs::Graph::from(&sg);
                // Preserve node positions
                for (src_i, sg_i) in &map {
                    if let (Some(src_node), Some(dst_node)) = (
                        src.g().node_weight(*src_i),
                        dst.g_mut().node_weight_mut(*sg_i),
                    ) {
                        dst_node.set_location(src_node.location());
                    }
                }
                self.g = DemoGraph::Undirected(dst);
                self.sync_counts();
            }
            (DemoGraph::Undirected(gu), true) => {
                // Undirected -> Directed (min->max)
                let src = gu.clone();
                let mut sg: petgraph::stable_graph::StableGraph<
                    (),
                    (),
                    petgraph::Directed,
                    DefaultIx,
                > = Default::default();
                let mut map: HashMap<NodeIndex, NodeIndex> = HashMap::new();
                for ni in src.g().node_indices() {
                    let new_ni = sg.add_node(());
                    map.insert(ni, new_ni);
                }
                for ei in src.g().edge_indices() {
                    if let Some((a, b)) = src.g().edge_endpoints(ei) {
                        let (u, v) = if a.index() <= b.index() {
                            (a, b)
                        } else {
                            (b, a)
                        };
                        let ai = map[&u];
                        let bi = map[&v];
                        let _ = sg.add_edge(ai, bi, ());
                    }
                }
                let mut dst: egui_graphs::Graph<(), (), petgraph::Directed, DefaultIx> =
                    egui_graphs::Graph::from(&sg);
                // Preserve node positions
                for (src_i, sg_i) in &map {
                    if let (Some(src_node), Some(dst_node)) = (
                        src.g().node_weight(*src_i),
                        dst.g_mut().node_weight_mut(*sg_i),
                    ) {
                        dst_node.set_location(src_node.location());
                    }
                }
                self.g = DemoGraph::Directed(dst);
                self.sync_counts();
            }
        }
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
        let mut g = generate_random_graph(
            self.settings_graph.count_node,
            self.settings_graph.count_edge,
        );
        Self::distribute_nodes_circle_generic(&mut g);
        self.g = DemoGraph::Directed(g);
        egui_graphs::reset::<FruchtermanReingoldWithCenterGravityState>(ui, None);
        egui_graphs::reset::<LayoutStateHierarchical>(ui, None);
        ui.ctx().set_visuals(egui::Visuals::dark());
        self.dark_mode = ui.ctx().style().visuals.dark_mode;
        #[cfg(feature = "events")]
        {
            self.last_events.clear();
            self.pan = [0.0, 0.0];
            self.zoom = 1.0;
            self.event_filters = EventFilters::default();
        }
        // Web: clear URL hash (remove g param and any others)
        #[cfg(target_arch = "wasm32")]
        {
            web_hash_clear();
        }
        self.metrics.reset();
    }

    pub fn distribute_nodes_circle_generic<Ty: petgraph::EdgeType>(
        g: &mut Graph<(), (), Ty, DefaultIx>,
    ) {
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
                    let mut st = egui_graphs::get_layout_state::<LayoutStateHierarchical>(ui, None);
                    st.triggered = false;
                    egui_graphs::set_layout_state::<LayoutStateHierarchical>(ui, st, None);
                }
            });

            ui.add_space(SECTION_SPACING);
            // Inline settings for the selected layout
            match self.selected_layout {
                DemoLayout::FruchtermanReingold => {
                    let mut state = egui_graphs::get_layout_state::<FruchtermanReingoldWithCenterGravityState>(ui, None);

                    // Animation section
                    CollapsingHeader::new("Animation").default_open(true).show(ui, |ui| {
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

                        ui.add_space(SECTION_SPACING);
                        ui.separator();
                        ui.horizontal(|ui| {
                            ui.label("Fast Forward");
                            info_icon(ui, "Advance the simulation instantly by a fixed number of steps or within a frame-time budget.");
                        });
                        ui.vertical(|ui| {
                            if ui.button("Fast-forward 100 steps").clicked() {
                                match &mut self.g {
                                    DemoGraph::Directed(g) => {
                                        egui_graphs::GraphView::<
                                            (), (), petgraph::Directed, petgraph::stable_graph::DefaultIx,
                                            egui_graphs::DefaultNodeShape, egui_graphs::DefaultEdgeShape,
                                            FruchtermanReingoldWithCenterGravityState,
                                            LayoutForceDirected<FruchtermanReingoldWithCenterGravity>,
                                        >::fast_forward_force_run(ui, g, 100, None);
                                        state = egui_graphs::get_layout_state::<FruchtermanReingoldWithCenterGravityState>(ui, None);
                                    }
                                    DemoGraph::Undirected(g) => {
                                        egui_graphs::GraphView::<
                                            (), (), petgraph::Undirected, petgraph::stable_graph::DefaultIx,
                                            egui_graphs::DefaultNodeShape, egui_graphs::DefaultEdgeShape,
                                            FruchtermanReingoldWithCenterGravityState,
                                            LayoutForceDirected<FruchtermanReingoldWithCenterGravity>,
                                        >::fast_forward_force_run(ui, g, 100, None);
                                        state = egui_graphs::get_layout_state::<FruchtermanReingoldWithCenterGravityState>(ui, None);
                                    }
                                }
                            }
                            if ui.button("Fast-forward 1000 steps_budgeted (100ms)").clicked() {
                                match &mut self.g {
                                    DemoGraph::Directed(g) => {
                                        let _ = egui_graphs::GraphView::<
                                            (), (), petgraph::Directed, petgraph::stable_graph::DefaultIx,
                                            egui_graphs::DefaultNodeShape, egui_graphs::DefaultEdgeShape,
                                            FruchtermanReingoldWithCenterGravityState,
                                            LayoutForceDirected<FruchtermanReingoldWithCenterGravity>,
                                        >::fast_forward_budgeted_force_run(ui, g, 1000, 100, None);
                                        state = egui_graphs::get_layout_state::<FruchtermanReingoldWithCenterGravityState>(ui, None);
                                    }
                                    DemoGraph::Undirected(g) => {
                                        let _ = egui_graphs::GraphView::<
                                            (), (), petgraph::Undirected, petgraph::stable_graph::DefaultIx,
                                            egui_graphs::DefaultNodeShape, egui_graphs::DefaultEdgeShape,
                                            FruchtermanReingoldWithCenterGravityState,
                                            LayoutForceDirected<FruchtermanReingoldWithCenterGravity>,
                                        >::fast_forward_budgeted_force_run(ui, g, 1000, 100, None);
                                        state = egui_graphs::get_layout_state::<FruchtermanReingoldWithCenterGravityState>(ui, None);
                                    }
                                }
                            }
                            if ui.button("Until stable (ε=0.01, ≤1000 steps)").clicked() {
                                match &mut self.g {
                                    DemoGraph::Directed(g) => {
                                        let _ = egui_graphs::GraphView::<
                                            (), (), petgraph::Directed, petgraph::stable_graph::DefaultIx,
                                            egui_graphs::DefaultNodeShape, egui_graphs::DefaultEdgeShape,
                                            FruchtermanReingoldWithCenterGravityState,
                                            LayoutForceDirected<FruchtermanReingoldWithCenterGravity>,
                                        >::fast_forward_until_stable_force_run(ui, g, 0.01, 1000, None);
                                        state = egui_graphs::get_layout_state::<FruchtermanReingoldWithCenterGravityState>(ui, None);
                                    }
                                    DemoGraph::Undirected(g) => {
                                        let _ = egui_graphs::GraphView::<
                                            (), (), petgraph::Undirected, petgraph::stable_graph::DefaultIx,
                                            egui_graphs::DefaultNodeShape, egui_graphs::DefaultEdgeShape,
                                            FruchtermanReingoldWithCenterGravityState,
                                            LayoutForceDirected<FruchtermanReingoldWithCenterGravity>,
                                        >::fast_forward_until_stable_force_run(ui, g, 0.01, 1000, None);
                                        state = egui_graphs::get_layout_state::<FruchtermanReingoldWithCenterGravityState>(ui, None);
                                    }
                                }
                            }
                            if ui.button("Until stable_budgeted (ε=0.01, ≤10000 steps, 1000ms)").clicked() {
                                match &mut self.g {
                                    DemoGraph::Directed(g) => {
                                        let _ = egui_graphs::GraphView::<
                                            (), (), petgraph::Directed, petgraph::stable_graph::DefaultIx,
                                            egui_graphs::DefaultNodeShape, egui_graphs::DefaultEdgeShape,
                                            FruchtermanReingoldWithCenterGravityState,
                                            LayoutForceDirected<FruchtermanReingoldWithCenterGravity>,
                                        >::fast_forward_until_stable_budgeted_force_run(ui, g, 0.01, 10000, 1000, None);
                                        state = egui_graphs::get_layout_state::<FruchtermanReingoldWithCenterGravityState>(ui, None);
                                    }
                                    DemoGraph::Undirected(g) => {
                                        let _ = egui_graphs::GraphView::<
                                            (), (), petgraph::Undirected, petgraph::stable_graph::DefaultIx,
                                            egui_graphs::DefaultNodeShape, egui_graphs::DefaultEdgeShape,
                                            FruchtermanReingoldWithCenterGravityState,
                                            LayoutForceDirected<FruchtermanReingoldWithCenterGravity>,
                                        >::fast_forward_until_stable_budgeted_force_run(ui, g, 0.01, 10000, 1000, None);
                                        state = egui_graphs::get_layout_state::<FruchtermanReingoldWithCenterGravityState>(ui, None);
                                    }
                                }
                            }
                        });
                    });

                    ui.add_space(SECTION_SPACING);

                    // Forces section
                    CollapsingHeader::new("Forces").default_open(true).show(ui, |ui| {
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

                        ui.add_space(SECTION_SPACING);
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
                    });

                    egui_graphs::set_layout_state::<FruchtermanReingoldWithCenterGravityState>(ui, state, None);
                }
                DemoLayout::Hierarchical => {
                    let mut state = egui_graphs::get_layout_state::<LayoutStateHierarchical>(ui, None);

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

                    egui_graphs::set_layout_state::<LayoutStateHierarchical>(ui, state, None);
                }
            }
        });
    }

    pub fn ui_layout_force_directed(&mut self, ui: &mut Ui) {
        let state = egui_graphs::get_layout_state::<FruchtermanReingoldWithCenterGravityState>(ui, None);

        egui_graphs::set_layout_state::<FruchtermanReingoldWithCenterGravityState>(ui, state, None);
    }

    pub fn ui_layout_hierarchical(&mut self, ui: &mut Ui) {
        let mut state = egui_graphs::get_layout_state::<LayoutStateHierarchical>(ui, None);

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

                ui.add_space(SECTION_SPACING);
                ui.horizontal(|ui| {
                    if ui.button("Re-run layout").clicked() {
                        state.triggered = false;
                    }
                    info_icon(ui, "Apply updated parameters and recompute positions once.");
                });
            });

        egui_graphs::set_layout_state::<LayoutStateHierarchical>(ui, state, None);
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
                ScrollArea::vertical()
                    .max_height(SELECTED_SCROLL_MAX_HEIGHT)
                    .show(ui, |ui| match &self.g {
                        DemoGraph::Directed(g) => {
                            for n in g.selected_nodes() {
                                ui.label(format!("{n:?}"));
                            }
                            for e in g.selected_edges() {
                                ui.label(format!("{e:?}"));
                            }
                        }
                        DemoGraph::Undirected(g) => {
                            for n in g.selected_nodes() {
                                ui.label(format!("{n:?}"));
                            }
                            for e in g.selected_edges() {
                                ui.label(format!("{e:?}"));
                            }
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
                #[cfg(feature = "events")]
                {
                    ui.set_min_height(crate::ui_consts::EVENTS_MIN_HEIGHT);
                }
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
        let (n, e) = match &self.g {
            DemoGraph::Directed(g) => (g.node_count(), g.edge_count()),
            DemoGraph::Undirected(g) => (g.node_count(), g.edge_count()),
        };
        self.settings_graph.count_node = n;
        self.settings_graph.count_edge = e;
    }

    // (moved) ui_import_tab in tabs::import_load
}

impl App for DemoApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Reset typing flag each frame; UI code will set it when a text field has focus
        self.typing_in_input = false;
        // Sync counts displayed on sliders with actual graph values
        self.sync_counts();

        // Handle global keyboard shortcuts and modal toggling
        self.process_keybindings(ctx);

        // Right side panel with controls
        if self.show_sidebar {
            egui::SidePanel::right("right")
                .default_width(SIDE_PANEL_WIDTH)
                .min_width(SIDE_PANEL_WIDTH)
                .show(ctx, |ui| {
                    // Tabs header using selectable labels
                    ui.horizontal(|ui| {
                        ui.selectable_value(
                            &mut self.right_tab,
                            RightTab::Playground,
                            "Playground",
                        );
                        ui.selectable_value(&mut self.right_tab, RightTab::Import, "Import/Load");
                    });
                    ui.separator();
                    match self.right_tab {
                        RightTab::Playground => self.ui_playground_tab(ui),
                        RightTab::Import => {
                            egui::ScrollArea::vertical().show(ui, |ui| self.ui_import_tab(ui));
                        }
                    }
                });
        }

        // Central graph view
        egui::CentralPanel::default().show(ctx, |ui| {
            if self.reset_requested {
                self.reset_all(ui);
                self.reset_requested = false;
            }
            // Graph rect (CentralPanel). Kept for overlay drawing below.
            let _max_rect = ui.max_rect();
            // Detect if any file is being dragged over the app. Some platforms/browsers
            // don't provide a pointer position during file drags, so don't require it.
            // We draw the overlay only within the CentralPanel rect anyway.
            self.drag_hover_graph = ctx.input(|i| !i.raw.hovered_files.is_empty());

            // Handle drops this frame (platform may provide bytes immediately or later). Process the first valid one.
            let mut maybe_text: Option<String> = None;
            let mut maybe_name: Option<String> = None;
            ctx.input(|i| {
                for f in &i.raw.dropped_files {
                    if let Some(bytes) = &f.bytes {
                        if let Ok(s) = std::str::from_utf8(bytes) {
                            maybe_text = Some(s.to_owned());
                            // Name (native path if available)
                            #[cfg(not(target_arch = "wasm32"))]
                            {
                                if let Some(path) = &f.path {
                                    if let Some(fname) = path.file_name().and_then(|o| o.to_str()) {
                                        maybe_name = Some(fname.to_owned());
                                    }
                                }
                            }
                            // Fallback: if no path or on web, use provided display name
                            if maybe_name.is_none() && !f.name.is_empty() {
                                maybe_name = Some(f.name.clone());
                            }
                            break;
                        }
                    }
                    // Native fallback: read from path if available
                    #[cfg(not(target_arch = "wasm32"))]
                    if let Some(path) = &f.path {
                        if let Ok(data) = std::fs::read(path) {
                            if let Ok(s) = String::from_utf8(data) {
                                maybe_text = Some(s);
                                if let Some(fname) = path.file_name().and_then(|o| o.to_str()) {
                                    maybe_name = Some(fname.to_owned());
                                }
                                break;
                            }
                        }
                    } else if maybe_name.is_none() && !f.name.is_empty() {
                        // If platform provided only the name (no path), keep it for the uploads list
                        maybe_name = Some(f.name.clone());
                    }
                    // On web or if no path/bytes-name captured yet, still try to capture the display name
                    if maybe_name.is_none() && !f.name.is_empty() {
                        maybe_name = Some(f.name.clone());
                    }
                }
            });
            if let Some(text) = maybe_text.take() {
                match crate::import::import_graph_from_str(&text) {
                    Ok(mut res) => {
                        let applied_positions = res.positions_applied;
                        match &mut res.g {
                            crate::import::ImportedGraph::Directed(g) => {
                                if !applied_positions {
                                    Self::distribute_nodes_circle_generic(g);
                                }
                                self.g = DemoGraph::Directed(g.clone());
                            }
                            crate::import::ImportedGraph::Undirected(g) => {
                                if !applied_positions {
                                    Self::distribute_nodes_circle_generic(g);
                                }
                                self.g = DemoGraph::Undirected(g.clone());
                            }
                        }
                        // If a layout state was imported, apply it next frame and switch UI to that layout
                        if let Some(pl) = res.pending_layout.take() {
                            self.pending_layout = Some(pl);
                            self.selected_layout = match self.pending_layout {
                                Some(spec::PendingLayout::FR(_)) => DemoLayout::FruchtermanReingold,
                                Some(spec::PendingLayout::Hier(_)) => DemoLayout::Hierarchical,
                                None => self.selected_layout,
                            };
                        }
                        self.sync_counts();
                        // Save to uploads list (cap to last 20)
                        let name = maybe_name
                            .unwrap_or_else(|| format!("Upload {}", self.user_uploads.len() + 1));
                        self.user_uploads.push(UserUpload {
                            name,
                            data: text.clone(),
                        });
                        if self.user_uploads.len() > 20 {
                            let overflow = self.user_uploads.len() - 20;
                            self.user_uploads.drain(0..overflow);
                        }
                        let (kind, n, e) = match &self.g {
                            DemoGraph::Directed(g) => ("directed", g.node_count(), g.edge_count()),
                            DemoGraph::Undirected(g) => {
                                ("undirected", g.node_count(), g.edge_count())
                            }
                        };
                        let suffix = if applied_positions {
                            " (positions applied)"
                        } else {
                            ""
                        };
                        self.status.push_success(format!(
                            "Loaded {} graph: {} nodes, {} edges{}",
                            kind, n, e, suffix
                        ));
                    }
                    Err(e) => {
                        self.status.push_error(format!("Drop error: {}", e));
                    }
                }
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

            match (&mut self.g, self.selected_layout) {
                (DemoGraph::Directed(ref mut g), DemoLayout::FruchtermanReingold) => {
                    if let Some(spec::PendingLayout::FR(st)) = self.pending_layout.take() {
                        egui_graphs::set_layout_state::<FruchtermanReingoldWithCenterGravityState>(ui, st, None);
                    }
                    let mut view = egui_graphs::GraphView::<
                        _,
                        _,
                        _,
                        _,
                        _,
                        _,
                        FruchtermanReingoldWithCenterGravityState,
                        LayoutForceDirected<FruchtermanReingoldWithCenterGravity>,
                    >::new(g)
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
                (DemoGraph::Undirected(ref mut g), DemoLayout::FruchtermanReingold) => {
                    if let Some(spec::PendingLayout::FR(st)) = self.pending_layout.take() {
                        egui_graphs::set_layout_state::<FruchtermanReingoldWithCenterGravityState>(ui, st, None);
                    }
                    let mut view = egui_graphs::GraphView::<
                        _,
                        _,
                        _,
                        _,
                        _,
                        _,
                        FruchtermanReingoldWithCenterGravityState,
                        LayoutForceDirected<FruchtermanReingoldWithCenterGravity>,
                    >::new(g)
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
                (DemoGraph::Directed(ref mut g), DemoLayout::Hierarchical) => {
                    if let Some(spec::PendingLayout::Hier(st)) = self.pending_layout.take() {
                        egui_graphs::set_layout_state::<LayoutStateHierarchical>(ui, st, None);
                    }
                    let mut view = egui_graphs::GraphView::<
                        _,
                        _,
                        _,
                        _,
                        _,
                        _,
                        LayoutStateHierarchical,
                        LayoutHierarchical,
                    >::new(g)
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
                (DemoGraph::Undirected(ref mut g), DemoLayout::Hierarchical) => {
                    if let Some(spec::PendingLayout::Hier(st)) = self.pending_layout.take() {
                        egui_graphs::set_layout_state::<LayoutStateHierarchical>(ui, st, None);
                    }
                    let mut view = egui_graphs::GraphView::<
                        _,
                        _,
                        _,
                        _,
                        _,
                        _,
                        LayoutStateHierarchical,
                        LayoutHierarchical,
                    >::new(g)
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

            // After rendering the view, handle pending one-shot navigation actions.

            // 1) Pan to graph center without changing zoom
            if self.pan_to_graph_pending {
                // TODO: do we need to interact with metadata explicitly here
                let mut meta = egui_graphs::Metadata::new("".to_string()).load(ui);
                let bounds = match &self.g {
                    DemoGraph::Directed(g) => g.bounds(),
                    DemoGraph::Undirected(g) => g.bounds(),
                };
                let graph_center = ((bounds.min.to_vec2() + bounds.max.to_vec2()) * 0.5).to_pos2();
                let new_pan = ui.max_rect().center().to_vec2() - graph_center.to_vec2() * meta.zoom;
                meta.pan = new_pan;
                meta.save(ui);
                self.pan_to_graph_pending = false;
                self.notify_info("Fit to screen (no zoom)");
            }

            // 2) Fit to screen once: turn off the auto-fit after it has just been applied.
            if self.fit_to_screen_once_pending && self.settings_navigation.fit_to_screen_enabled {
                self.settings_navigation.fit_to_screen_enabled = false;
                self.fit_to_screen_once_pending = false;
            }

            #[cfg(feature = "events")]
            self.consume_events();

            // Capture latest layout step count for overlay display
            if let DemoLayout::FruchtermanReingold = self.selected_layout {
                let steps = match &self.g {
                    DemoGraph::Directed(_) => {
                        let st = egui_graphs::get_layout_state::<FruchtermanReingoldWithCenterGravityState>(ui, None);
                        st.base.step_count as usize
                    }
                    DemoGraph::Undirected(_) => {
                        let st = egui_graphs::get_layout_state::<FruchtermanReingoldWithCenterGravityState>(ui, None);
                        st.base.step_count as usize
                    }
                };
                self.metrics.set_last_step_count(steps);
            } else {
                self.metrics.set_last_step_count(0);
            }

            // Record performance samples for 5s rolling average
            self.record_perf_sample(ui);
            // Small info overlay (top-left): version + source link
            overlays::info_overlay::render_info_overlay(ui);
            // If last drop produced an error, surface it unobtrusively
            // Status line with auto timeouts (top-left)
            self.status.retain_active();
            let pos = egui::pos2(UI_MARGIN, UI_MARGIN + 22.0);
            if let Some(m) = self.status.latest() {
                let font = egui::TextStyle::Monospace.resolve(ui.style());
                let color = match m.kind {
                    StatusKind::Error => ui.visuals().error_fg_color,
                    StatusKind::Success => Color32::from_rgb(80, 200, 120),
                    StatusKind::Info => ui.visuals().hyperlink_color,
                };
                ui.painter()
                    .text(pos, Align2::LEFT_TOP, m.text.clone(), font, color);
            }
            // Draw overlay inside the CentralPanel so it stays within the graph area
            if self.show_debug_overlay {
                let (n, e) = match &self.g {
                    DemoGraph::Directed(g) => (g.node_count(), g.edge_count()),
                    DemoGraph::Undirected(g) => (g.node_count(), g.edge_count()),
                };
                #[cfg(feature = "events")]
                let (pan_opt, zoom_opt) = (Some(self.pan), Some(self.zoom));
                #[cfg(not(feature = "events"))]
                let (pan_opt, zoom_opt) = (None, None);
                crate::overlays::debug_overlay::render(
                    ui,
                    &self.metrics,
                    n,
                    e,
                    self.metrics.last_step_count(),
                    pan_opt,
                    zoom_opt,
                );
            }
            // Draw drag-drop hint last so it's visible above the graph
            if self.drag_hover_graph {
                draw_drop_overlay(ui, ui.max_rect());
            }
            // Draw toggle button for the side panel at bottom-right of the graph area
            self.overlay_toggle_sidebar_button(ui);
            // Bottom tips removed; use single-line version/source overlay instead (moved below)
        });

        self.update_fps();

        // Draw modal after main UI
        self.keybindings_modal(ctx);
    }
}

// Small helper methods for consistent status notifications across actions
impl DemoApp {
    fn notify_info(&mut self, msg: impl Into<String>) {
        self.status.push_info(msg);
    }

    fn notify_added(&mut self, singular: &str, plural: &str, n: u32) {
        let text = if n == 1 {
            format!("+1 {}", singular)
        } else {
            format!("+{} {}", n, plural)
        };
        self.status.push_success(text);
    }

    fn notify_removed(&mut self, singular: &str, plural: &str, n: u32) {
        let text = if n == 1 {
            format!("-1 {}", singular)
        } else {
            format!("-{} {}", n, plural)
        };
        self.status.push_success(text);
    }

    fn notify_swapped(&mut self, singular: &str, plural: &str, n: u32) {
        let text = if n == 1 {
            format!("Swap 1 {}", singular)
        } else {
            format!("Swap {} {}", n, plural)
        };
        self.status.push_success(text);
    }
}

impl DemoApp {
    // Minimal schema to load JSON graphs: {"nodes":[id...],"edges":[[source,target],...]}
    // ids are integers; node payload and edge payload are ignored (())
    // On web, file bytes are provided by egui; no filesystem access needed.

    // moved: ui_import_tab in tabs::import_load

    fn record_perf_sample(&mut self, ui: &mut egui::Ui) {
        let route = pick_metrics_route(&self.g, self.selected_layout);
        let (step_ms, draw_ms) = match route {
            MetricsRoute::DirectedFR => egui_graphs::get_metrics(ui, None),
            MetricsRoute::UndirectedFR => egui_graphs::get_metrics(ui, None),
            MetricsRoute::DirectedHier => egui_graphs::get_metrics(ui, None),
            MetricsRoute::UndirectedHier => egui_graphs::get_metrics(ui, None),
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

        #[cfg(test)]
        mod tests {
            use super::*;
            use egui_graphs::Graph;
            use petgraph::{stable_graph::DefaultIx, Directed, Undirected};

            #[test]
            fn picks_metrics_route_by_graph_type_and_layout() {
                // Directed graph
                let sg_d: petgraph::stable_graph::StableGraph<(), (), Directed, DefaultIx> =
                    Default::default();
                let g_d: Graph<(), (), Directed, DefaultIx> = Graph::from(&sg_d);
                let demo_d = DemoGraph::Directed(g_d);
                assert_eq!(
                    pick_metrics_route(&demo_d, DemoLayout::FruchtermanReingold),
                    MetricsRoute::DirectedFR
                );
                assert_eq!(
                    pick_metrics_route(&demo_d, DemoLayout::Hierarchical),
                    MetricsRoute::DirectedHier
                );

                // Undirected graph
                let sg_u: petgraph::stable_graph::StableGraph<(), (), Undirected, DefaultIx> =
                    Default::default();
                let g_u: Graph<(), (), Undirected, DefaultIx> = Graph::from(&sg_u);
                let demo_u = DemoGraph::Undirected(g_u);
                assert_eq!(
                    pick_metrics_route(&demo_u, DemoLayout::FruchtermanReingold),
                    MetricsRoute::UndirectedFR
                );
                assert_eq!(
                    pick_metrics_route(&demo_u, DemoLayout::Hierarchical),
                    MetricsRoute::UndirectedHier
                );
            }
        }
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
                ui.label(RichText::new(title).strong().color(accent).size(HEADING_TEXT_SIZE));
                egui::Grid::new(grid_id).num_columns(2).spacing(egui::vec2(8.0, 4.0)).show(ui, |ui| {
                    for (key, desc) in entries {
                        ui.code(*key);
                        ui.label(*desc);
                        ui.end_row();
                    }
                });
                    ui.add_space(SECTION_SPACING);
            };

            // Graph elements
            render_group(ui, "Graph elements",
                &[
                    ("n", "+1 node"),
                    ("Shift+n", "-1 node"),
                    ("Ctrl+Shift+n", "swap 1 node"),
                    ("m", "+10 nodes (up to max)"),
                    ("Shift+m", "-10 nodes"),
                    ("Ctrl+Shift+m", "swap 10 nodes"),
                    ("e", "+1 edge"),
                    ("Shift+e", "-1 edge"),
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
                    ("Backspace", "reset all"),
            ("Ctrl+Space", "toggle zoom & pan / fit to screen"),
            ("Ctrl+Shift+Space", "Fit to screen (no zoom)"),
            ("Space", "Fit to screen"),
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
        let btn_size = egui::vec2(OVERLAY_BTN_SIZE, OVERLAY_BTN_SIZE);
        let spacing = OVERLAY_BTN_SPACING;
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
                let help_text = egui::RichText::new("ℹ").size(OVERLAY_ICON_SIZE);
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
                let arrow_text = egui::RichText::new(arrow).size(OVERLAY_ICON_SIZE);
                let response = ui_area.add_sized(btn_size, egui::Button::new(arrow_text));
                if response.on_hover_text(tip).clicked() {
                    self.show_sidebar = !self.show_sidebar;
                }
            });
    }
    // Bottom instructional tips removed
    fn process_keybindings(&mut self, ctx: &egui::Context) {
        // Tab should always toggle the sidebar and never move focus.
        // Handle it unconditionally and consume Tab/Shift+Tab every frame.
        let mut toggle = false;
        ctx.input(|i| {
            if i.key_pressed(egui::Key::Tab) && !i.modifiers.any() {
                toggle = true;
            }
        });
        // Consume Tab / Shift+Tab and strip Tab events to disable focus traversal
        let shifted = egui::Modifiers {
            shift: true,
            ..egui::Modifiers::default()
        };
        ctx.input_mut(|i| {
            let _ = i.consume_key(egui::Modifiers::default(), egui::Key::Tab);
            let _ = i.consume_key(shifted, egui::Key::Tab);
            i.events.retain(|ev| match ev {
                egui::Event::Key {
                    key: egui::Key::Tab,
                    ..
                } => false,
                egui::Event::Text(t) if t == "\t" => false,
                _ => true,
            });
        });
        if toggle {
            self.show_sidebar = !self.show_sidebar;
        }

        // Other keybindings (still disabled while typing to avoid graph actions while editing text)
        let typing = self.typing_in_input || ctx.wants_keyboard_input();
        let cmds = if typing {
            Vec::new()
        } else {
            dispatch_keybindings(ctx)
        };
        let mut open_modal = false;
        let mut close_modal = false;
        for c in cmds {
            match c {
                Command::ToggleDebug => {
                    self.show_debug_overlay = !self.show_debug_overlay;
                    self.notify_info("Toggled debug overlay");
                }
                Command::OpenKeybindings => {
                    if self.show_keybindings_overlay {
                        close_modal = true;
                        self.notify_info("Closed keybindings");
                    } else {
                        open_modal = true;
                        self.notify_info("Opened keybindings");
                    }
                }
                Command::CloseKeybindings => {
                    self.show_keybindings_overlay = false;
                    self.notify_info("Closed keybindings");
                }
                Command::ResetAll => {
                    self.reset_requested = true;
                    self.notify_info("Reset all");
                }
                Command::ToggleNavMode => {
                    // Switch zoom&pan and fit_to_screen (mutually exclusive)
                    let enable_zoom_pan = !self.settings_navigation.zoom_and_pan_enabled;
                    self.settings_navigation.zoom_and_pan_enabled = enable_zoom_pan;
                    self.settings_navigation.fit_to_screen_enabled = !enable_zoom_pan;
                    if enable_zoom_pan {
                        self.notify_info("Toggle pan and zoom");
                    } else {
                        self.notify_info("Toggle fit to screen");
                    }
                }
                Command::FitToScreenOnce => {
                    // Enable fit_to_screen for a single frame then disable it after draw.
                    // If already enabled, this is a noop per requirement.
                    if !self.settings_navigation.fit_to_screen_enabled {
                        self.settings_navigation.fit_to_screen_enabled = true;
                        self.fit_to_screen_once_pending = true;
                        self.notify_info("Fit to screen");
                    }
                }
                Command::PanToGraph => {
                    // Request a one-off pan to graph center; will be applied after draw this frame.
                    self.pan_to_graph_pending = true;
                }
                Command::AddNodes(n) => {
                    GraphActions { g: &mut self.g }.add_nodes(n);
                    self.notify_added("node", "nodes", n);
                }
                Command::RemoveNodes(n) => {
                    GraphActions { g: &mut self.g }.remove_nodes(n);
                    self.notify_removed("node", "nodes", n);
                }
                Command::SwapNodes(n) => {
                    GraphActions { g: &mut self.g }.swap_nodes(n);
                    self.notify_swapped("node", "nodes", n);
                }
                Command::AddEdges(n) => {
                    GraphActions { g: &mut self.g }.add_edges(n);
                    self.notify_added("edge", "edges", n);
                }
                Command::RemoveEdges(n) => {
                    GraphActions { g: &mut self.g }.remove_edges(n);
                    self.notify_removed("edge", "edges", n);
                }
                Command::SwapEdges(n) => {
                    GraphActions { g: &mut self.g }.swap_edges(n);
                    self.notify_swapped("edge", "edges", n);
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

fn draw_drop_overlay(ui: &mut egui::Ui, rect: Rect) {
    let fade = Color32::from_black_alpha(140);
    ui.painter().rect_filled(rect, 0.0, fade);

    let txt = "Drop JSON to load graph";
    let font = egui::TextStyle::Heading.resolve(ui.style());
    ui.painter().text(
        rect.center(),
        Align2::CENTER_CENTER,
        txt,
        font,
        Color32::WHITE,
    );
}

// --- Web-only helpers for URL hash params and bundled example lookup ---
#[cfg(target_arch = "wasm32")]
pub(crate) fn web_hash_get_param(key: &str) -> Option<String> {
    let window = web_sys::window()?;
    let loc = window.location();
    let hash = loc.hash().ok().unwrap_or_default();
    let raw = hash.strip_prefix('#').unwrap_or(hash.as_str());
    if raw.is_empty() {
        return None;
    }
    for pair in raw.split('&') {
        if pair.is_empty() {
            continue;
        }
        let mut it = pair.splitn(2, '=');
        let k = it.next().unwrap_or("");
        let v = it.next().unwrap_or("");
        if k == key {
            let decoded = js_sys::decode_uri_component(v)
                .ok()
                .and_then(|js| js.as_string())
                .unwrap_or_else(|| v.to_string());
            return Some(decoded);
        }
    }
    None
}

#[cfg(target_arch = "wasm32")]
pub(crate) fn web_hash_set_param(key: &str, value: &str) {
    let window = match web_sys::window() {
        Some(w) => w,
        None => return,
    };
    let loc = window.location();
    let hash = loc.hash().unwrap_or_default();
    let raw = hash.strip_prefix('#').unwrap_or(hash.as_str());
    let mut params: std::collections::BTreeMap<String, String> = std::collections::BTreeMap::new();
    if !raw.is_empty() {
        for pair in raw.split('&') {
            if pair.is_empty() {
                continue;
            }
            let mut it = pair.splitn(2, '=');
            let k = it.next().unwrap_or("");
            let v = it.next().unwrap_or("");
            if !k.is_empty() {
                params.insert(k.to_string(), v.to_string());
            }
        }
    }
    params.insert(key.to_string(), value.to_string());
    let new_hash_body = params
        .iter()
        .map(|(k, v)| format!("{}={}", k, v))
        .collect::<Vec<_>>()
        .join("&");

    // Update hash via set_hash (works across browsers without extra web-sys features)
    let _ = loc.set_hash(&new_hash_body);
}

#[cfg(target_arch = "wasm32")]
pub(crate) fn web_lookup_example_asset(name: &str) -> Option<&'static str> {
    #[allow(non_upper_case_globals)]
    mod assets_manifest_for_hash {
        include!(concat!(env!("OUT_DIR"), "/assets_manifest.rs"));
    }
    for (n, d) in assets_manifest_for_hash::ASSETS.iter() {
        if *n == name {
            return Some(*d);
        }
    }
    None
}

// Build full page URL preserving current hash
#[cfg(target_arch = "wasm32")]
pub(crate) fn web_build_share_url_current() -> Option<String> {
    let window = web_sys::window()?;
    window.location().href().ok()
}

// Build a shareable URL for a specific example (set g=name in hash and return href)
#[cfg(target_arch = "wasm32")]
pub(crate) fn web_build_share_url_for_example(name: &str) -> Option<String> {
    let window = web_sys::window()?;
    let loc = window.location();
    let href = loc.href().ok()?;
    let base = match href.split_once('#') {
        Some((b, _)) => b.to_string(),
        None => href.clone(),
    };
    // Parse existing hash params without mutating the current URL
    let hash = loc.hash().unwrap_or_default();
    let raw = hash.strip_prefix('#').unwrap_or(hash.as_str());
    let mut params: std::collections::BTreeMap<String, String> = std::collections::BTreeMap::new();
    if !raw.is_empty() {
        for pair in raw.split('&') {
            if pair.is_empty() {
                continue;
            }
            let mut it = pair.splitn(2, '=');
            let k = it.next().unwrap_or("");
            let v = it.next().unwrap_or("");
            if !k.is_empty() {
                params.insert(k.to_string(), v.to_string());
            }
        }
    }
    params.insert("g".to_string(), name.to_string());
    let new_hash_body = params
        .iter()
        .map(|(k, v)| format!("{}={}", k, v))
        .collect::<Vec<_>>()
        .join("&");
    Some(format!("{}#{}", base, new_hash_body))
}

// Clear the URL hash entirely (remove all params)
#[cfg(target_arch = "wasm32")]
pub(crate) fn web_hash_clear() {
    if let Some(window) = web_sys::window() {
        let _ = window.location().set_hash("");
    }
}
