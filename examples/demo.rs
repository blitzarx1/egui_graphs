use core::cmp::Ordering;
use eframe::{run_native, App, CreationContext};
use egui::{self, Align2, CollapsingHeader, Pos2, ScrollArea, Ui};
use egui_graphs::{generate_random_graph, Graph, LayoutForceDirected, LayoutStateForceDirected};
use petgraph::stable_graph::{DefaultIx, EdgeIndex, NodeIndex};
use petgraph::Directed;
use rand::Rng;
use std::time::{Duration, Instant};

#[cfg(feature = "events")]
use crossbeam::channel::{unbounded, Receiver, Sender};
#[cfg(feature = "events")]
use egui_graphs::events::Event;

mod settings_local {
    #[derive(Default)]
    pub struct SettingsInteraction {
        pub dragging_enabled: bool,
        pub node_clicking_enabled: bool,
        pub node_selection_enabled: bool,
        pub node_selection_multi_enabled: bool,
        pub edge_clicking_enabled: bool,
        pub edge_selection_enabled: bool,
        pub edge_selection_multi_enabled: bool,
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
        pub screen_padding: f32,
    }
    impl Default for SettingsNavigation {
        fn default() -> Self {
            Self {
                fit_to_screen_enabled: true,
                zoom_and_pan_enabled: false,
                zoom_speed: 0.1,
                screen_padding: 0.3,
            }
        }
    }

    pub struct SettingsGraph {
        pub count_node: usize,
        pub count_edge: usize,
    }
    impl Default for SettingsGraph {
        fn default() -> Self {
            Self {
                count_node: 25,
                count_edge: 50,
            }
        }
    }
}
use settings_local as settings;

fn info_icon(ui: &mut egui::Ui, tip: &str) {
    ui.add_space(4.0);
    ui.small_button("ℹ").on_hover_text(tip);
}

mod drawers {
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
            ui.add(egui::Slider::new(&mut v.nodes, 0..=2500));
            if ui.small_button("-10").clicked() {
                v.nodes = v.nodes.saturating_sub(10);
            }
            if ui.small_button("-1").clicked() {
                v.nodes = v.nodes.saturating_sub(1);
            }
            if ui.small_button("+1").clicked() {
                v.nodes = (v.nodes + 1).min(2500);
            }
            if ui.small_button("+10").clicked() {
                v.nodes = (v.nodes + 10).min(2500);
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
            ui.add(egui::Slider::new(&mut v.edges, 0..=2500));
            if ui.small_button("-10").clicked() {
                v.edges = v.edges.saturating_sub(10);
            }
            if ui.small_button("-1").clicked() {
                v.edges = v.edges.saturating_sub(1);
            }
            if ui.small_button("+1").clicked() {
                v.edges = (v.edges + 1).min(2500);
            }
            if ui.small_button("+10").clicked() {
                v.edges = (v.edges + 10).min(2500);
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

#[cfg(feature = "events")]
const EVENTS_LIMIT: usize = 1000;

pub struct DemoApp {
    g: Graph<(), (), Directed, DefaultIx>,
    settings_graph: settings::SettingsGraph,
    settings_interaction: settings::SettingsInteraction,
    settings_navigation: settings::SettingsNavigation,
    settings_style: settings::SettingsStyle,
    fps: f32,
    last_update_time: Instant,
    frames_last_time_span: usize,
    #[cfg(not(feature = "events"))]
    copy_tip_until: Option<Instant>,
    #[cfg(feature = "events")]
    pan: [f32; 2],
    #[cfg(feature = "events")]
    zoom: f32,
    #[cfg(feature = "events")]
    last_events: Vec<String>,
    #[cfg(feature = "events")]
    event_publisher: Sender<Event>,
    #[cfg(feature = "events")]
    event_consumer: Receiver<Event>,
    dark_mode: bool,
}

impl DemoApp {
    fn new(cc: &CreationContext<'_>) -> Self {
        let settings_graph = settings::SettingsGraph::default();
        let mut g = generate_random_graph(settings_graph.count_node, settings_graph.count_edge);
        Self::distribute_nodes_circle(&mut g);

        #[cfg(feature = "events")]
        let (event_publisher, event_consumer) = unbounded();

        Self {
            g,
            settings_graph,
            settings_interaction: settings::SettingsInteraction::default(),
            settings_navigation: settings::SettingsNavigation::default(),
            settings_style: settings::SettingsStyle {
                labels_always: false,
                edge_deemphasis: true,
            },
            fps: 0.0,
            last_update_time: Instant::now(),
            frames_last_time_span: 0,
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
            dark_mode: cc.egui_ctx.style().visuals.dark_mode,
        }
    }

    fn random_node_idx(&self) -> Option<NodeIndex> {
        let cnt = self.g.node_count();
        if cnt == 0 {
            return None;
        }
        let idx = rand::rng().random_range(0..cnt);
        self.g.g().node_indices().nth(idx)
    }
    fn random_edge_idx(&self) -> Option<EdgeIndex> {
        let cnt = self.g.edge_count();
        if cnt == 0 {
            return None;
        }
        let idx = rand::rng().random_range(0..cnt);
        self.g.g().edge_indices().nth(idx)
    }
    fn add_random_node(&mut self) {
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
    fn remove_random_node(&mut self) {
        if let Some(i) = self.random_node_idx() {
            self.remove_node(i);
        }
    }
    fn remove_node(&mut self, idx: NodeIndex) {
        self.g.remove_node(idx);
        self.settings_graph.count_edge = self.g.edge_count();
    }
    fn add_random_edge(&mut self) {
        if let (Some(a), Some(b)) = (self.random_node_idx(), self.random_node_idx()) {
            self.add_edge(a, b);
        }
    }
    fn add_edge(&mut self, a: NodeIndex, b: NodeIndex) {
        self.g.add_edge(a, b, ());
    }
    fn remove_random_edge(&mut self) {
        if let Some(eidx) = self.random_edge_idx() {
            if let Some((a, b)) = self.g.edge_endpoints(eidx) {
                self.remove_edge(a, b);
            }
        }
    }
    fn remove_edge(&mut self, a: NodeIndex, b: NodeIndex) {
        let edge_id_opt = { self.g.edges_connecting(a, b).map(|(eid, _)| eid).next() };
        if let Some(edge_id) = edge_id_opt {
            self.g.remove_edge(edge_id);
        }
    }

    fn update_fps(&mut self) {
        self.frames_last_time_span += 1;
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_update_time);
        if elapsed.as_secs() >= 1 {
            self.last_update_time = now;
            self.fps = self.frames_last_time_span as f32 / elapsed.as_secs_f32();
            self.frames_last_time_span = 0;
        }
    }

    #[cfg(feature = "events")]
    fn handle_events(&mut self) {
        self.event_consumer.try_iter().for_each(|e| {
            if self.last_events.len() > EVENTS_LIMIT {
                self.last_events.remove(0);
            }
            self.last_events.push(format!("{e:?}"));
            match e {
                Event::Pan(p) => self.pan = p.new_pan,
                Event::Zoom(p) => self.zoom = p.new_zoom,
                _ => {}
            }
        });
    }

    fn ui_graph_section(&mut self, ui: &mut Ui) {
        drawers::graph_count_sliders(
            ui,
            drawers::GraphCountSliders {
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

    fn reset_all(&mut self, ui: &mut Ui) {
        self.settings_graph = settings::SettingsGraph::default();
        self.settings_interaction = settings::SettingsInteraction::default();
        self.settings_navigation = settings::SettingsNavigation::default();
        self.settings_style = settings::SettingsStyle {
            labels_always: false,
            edge_deemphasis: true,
        };
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
            LayoutStateForceDirected,
            LayoutForceDirected,
        >::reset(ui);
        ui.ctx().set_visuals(egui::Visuals::dark());
        self.dark_mode = ui.ctx().style().visuals.dark_mode;
        #[cfg(feature = "events")]
        {
            self.last_events.clear();
            self.pan = [0.0, 0.0];
            self.zoom = 1.0;
        }
        self.fps = 0.0;
    }

    fn distribute_nodes_circle(g: &mut Graph<(), (), Directed, DefaultIx>) {
        let n = g.node_count().max(1);
        if n == 0 {
            return;
        }
        let radius = (n as f32).sqrt() * 50.0 + 50.0;
        let indices: Vec<_> = g.g().node_indices().collect();
        for (i, idx) in indices.into_iter().enumerate() {
            if let Some(node) = g.g_mut().node_weight_mut(idx) {
                let angle = i as f32 / n as f32 * std::f32::consts::TAU;
                node.set_location(Pos2::new(radius * angle.cos(), radius * angle.sin()));
            }
        }
    }

    fn ui_navigation(&mut self, ui: &mut Ui) {
        CollapsingHeader::new("Navigation")
            .default_open(true)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    if ui
                        .checkbox(
                            &mut self.settings_navigation.fit_to_screen_enabled,
                            "fit_to_screen",
                        )
                        .clicked()
                        && self.settings_navigation.fit_to_screen_enabled
                    {
                        self.settings_navigation.zoom_and_pan_enabled = false;
                    }
                    info_icon(ui, "Continuously recompute zoom/pan so whole graph stays visible.");
                });
                ui.add_enabled_ui(!self.settings_navigation.fit_to_screen_enabled, |ui| {
                    ui.horizontal(|ui| {
                        ui.checkbox(
                            &mut self.settings_navigation.zoom_and_pan_enabled,
                            "zoom_and_pan",
                        );
                        info_icon(ui, "Manual navigation: Ctrl+wheel (zoom), drag (pan / node drag). Disable if auto-fit.");
                    });
                });
                ui.horizontal(|ui| {
                    ui.add(
                        egui::Slider::new(&mut self.settings_navigation.zoom_speed, 0.01..=1.0)
                            .text("zoom_speed"),
                    );
                    info_icon(ui, "Multiplier controlling how fast zoom changes per wheel step.");
                });
                ui.add_enabled_ui(self.settings_navigation.fit_to_screen_enabled, |ui| {
                    ui.horizontal(|ui| {
                        ui.add(
                            egui::Slider::new(&mut self.settings_navigation.screen_padding, 0.0..=1.0)
                                .text("screen_padding"),
                        );
                        info_icon(ui, "Extra fractional padding around graph when auto-fitting (0 = tight fit, 0.3 = 30% larger).");
                    });
                });
            });
    }

    #[allow(clippy::unused_self)]
    fn ui_layout_force_directed(&mut self, ui: &mut Ui) {
        let mut state = egui_graphs::GraphView::<
            (),
            (),
            petgraph::Directed,
            petgraph::stable_graph::DefaultIx,
            egui_graphs::DefaultNodeShape,
            egui_graphs::DefaultEdgeShape,
            LayoutStateForceDirected,
            LayoutForceDirected,
        >::get_layout_state(ui);

        CollapsingHeader::new("Force Directed Layout")
            .default_open(true)
            .show(ui, |ui| {
                fn info_icon(ui: &mut egui::Ui, tip: &str) {
                    ui.add_space(4.0);
                    if ui.small_button("ℹ").on_hover_text(tip).clicked() {
                    }
                }

                ui.horizontal(|ui| {
                    ui.checkbox(&mut state.is_running, "running");
                    info_icon(ui, "Run/pause the simulation. When paused node positions stay fixed.");
                });
                ui.horizontal(|ui| {
                    ui.add(egui::Slider::new(&mut state.dt, 0.001..=0.2).text("dt"));
                    info_icon(ui, "Integration time step (Euler). Larger = faster movement but less stable.");
                });
                ui.horizontal(|ui| {
                    ui.add(egui::Slider::new(&mut state.damping, 0.0..=1.0).text("damping"));
                    info_icon(ui, "Velocity damping per frame. 1 = no damping, 0 = immediate stop.");
                });
                ui.horizontal(|ui| {
                    ui.add(egui::Slider::new(&mut state.max_step, 0.1..=50.0).text("max_step"));
                    info_icon(ui, "Maximum pixel displacement applied per frame to prevent explosions.");
                });
                ui.horizontal(|ui| {
                    ui.add(egui::Slider::new(&mut state.gravity_base, 100.0..=2500.0).text("gravity_base"));
                    info_icon(ui, "Base strength of gentle pull toward canvas center (scaled inversely by view size).");
                });
                ui.horizontal(|ui| {
                    ui.add(egui::Slider::new(&mut state.epsilon, 1e-5..=1e-1).logarithmic(true).text("epsilon"));
                    info_icon(ui, "Minimum distance clamp to avoid division by zero in force calculations.");
                });
            });

        egui_graphs::GraphView::<
            (),
            (),
            petgraph::Directed,
            petgraph::stable_graph::DefaultIx,
            egui_graphs::DefaultNodeShape,
            egui_graphs::DefaultEdgeShape,
            LayoutStateForceDirected,
            LayoutForceDirected,
        >::set_layout_state(ui, state);
    }

    fn ui_interaction(&mut self, ui: &mut Ui) {
        CollapsingHeader::new("Interaction")
            .default_open(true)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.checkbox(
                        &mut self.settings_interaction.dragging_enabled,
                        "dragging_enabled",
                    );
                    info_icon(ui, "Drag nodes with pointer when enabled.");
                });
                ui.horizontal(|ui| {
                    ui.checkbox(
                        &mut self.settings_interaction.node_clicking_enabled,
                        "node_clicking",
                    );
                    info_icon(
                        ui,
                        "Enable click events for nodes (required for selection).",
                    );
                });
                ui.add_enabled_ui(
                    !self.settings_interaction.node_selection_multi_enabled,
                    |ui| {
                        ui.horizontal(|ui| {
                            if ui
                                .checkbox(
                                    &mut self.settings_interaction.node_selection_enabled,
                                    "node_selection",
                                )
                                .clicked()
                                && self.settings_interaction.node_selection_enabled
                            {
                                self.settings_interaction.node_clicking_enabled = true;
                            }
                            info_icon(ui, "Single node selection on click.");
                        });
                    },
                );
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
                    }
                    info_icon(ui, "Allow multiple nodes selected.");
                });
                ui.add_enabled_ui(
                    !(self.settings_interaction.edge_selection_enabled
                        || self.settings_interaction.edge_selection_multi_enabled),
                    |ui| {
                        ui.horizontal(|ui| {
                            ui.checkbox(
                                &mut self.settings_interaction.edge_clicking_enabled,
                                "edge_clicking",
                            );
                            info_icon(
                                ui,
                                "Enable click events for edges (required for selection).",
                            );
                        });
                    },
                );
                ui.add_enabled_ui(
                    !self.settings_interaction.edge_selection_multi_enabled,
                    |ui| {
                        ui.horizontal(|ui| {
                            if ui
                                .checkbox(
                                    &mut self.settings_interaction.edge_selection_enabled,
                                    "edge_selection",
                                )
                                .clicked()
                                && self.settings_interaction.edge_selection_enabled
                            {
                                self.settings_interaction.edge_clicking_enabled = true;
                            }
                            info_icon(ui, "Single edge selection on click.");
                        });
                    },
                );
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
                    }
                    info_icon(ui, "Allow multiple edges selected.");
                });
            });
    }

    fn ui_style(&mut self, ui: &mut Ui) {
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
            // Labels toggle line
            ui.horizontal(|ui| {
                ui.checkbox(&mut self.settings_style.labels_always, "labels_always");
                info_icon(
                    ui,
                    "Always render node & edge labels instead of only on interaction.",
                );
            });
            ui.add_space(2.0);
            // Edge deemphasis line
            ui.horizontal(|ui| {
                ui.checkbox(&mut self.settings_style.edge_deemphasis, "edge_deemphasis");
                info_icon(ui, "Dim non-selected edges to highlight current selection.");
            });
        });
    }

    fn ui_selected(&mut self, ui: &mut Ui) {
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

    #[allow(clippy::unused_self)]
    fn ui_events(&mut self, ui: &mut Ui) {
        CollapsingHeader::new("Last Events")
            .default_open(true)
            .show(ui, |ui| {
                #[cfg(feature = "events")]
                {
                    if ui.button("clear").clicked() {
                        self.last_events.clear();
                    }
                    ScrollArea::vertical().max_height(200.0).show(ui, |ui| {
                        for ev in self.last_events.iter().rev() {
                            ui.label(ev);
                        }
                    });
                }
                #[cfg(not(feature = "events"))]
                ui.label("Re-run with --features events to see interaction events");
            });
    }

    fn overlay_debug(&self, ctx: &egui::Context) {
        use egui::{Area, RichText};
        let text = {
            let fps_line = format!("FPS: {:.1}", self.fps);
            let n_line = format!("N: {}", self.g.node_count());
            let e_line = format!("E: {}", self.g.edge_count());
            #[cfg(feature = "events")]
            let zoom_line = format!("Zoom: {:.3}", self.zoom);
            #[cfg(feature = "events")]
            let pan_line = format!("Pan: [{:.1},{:.1}]", self.pan[0], self.pan[1]);
            #[cfg(not(feature = "events"))]
            let zoom_line = "Zoom: enable events feature".to_string();
            #[cfg(not(feature = "events"))]
            let pan_line = "Pan: enable events feature".to_string();
            format!("{fps_line}\n{n_line}\n{e_line}\n{zoom_line}\n{pan_line}")
        };

        let visuals = &ctx.style().visuals;
        Area::new(egui::Id::new("debug_overlay"))
            .movable(false)
            .interactable(false)
            .anchor(Align2::LEFT_TOP, [10.0, 10.0])
            .show(ctx, |ui| {
                ui.label(
                    RichText::new(text)
                        .monospace()
                        .color(visuals.strong_text_color())
                        .size(14.0),
                );
            });
    }

    #[cfg(not(feature = "events"))]
    fn show_events_feature_tip(&mut self, ui: &mut Ui) {
        ui.group(|ui| {
            ui.colored_label(
				egui::Color32::from_rgb(200, 180, 40),
				"Tip: enable the 'events' feature to see interaction events (pan/zoom, clicks, selections).",
			);
            let cmd = "cargo run --example demo --features events";
            ui.horizontal(|ui| {
                ui.code(cmd);
                if ui
                    .small_button("📋")
                    .on_hover_text("Copy command")
                    .clicked()
                {
                    ui.ctx().copy_text(cmd.to_string());
                    self.copy_tip_until = Some(Instant::now() + Duration::from_millis(1500));
                    ui.ctx().request_repaint();
                }
            });
            if let Some(until) = self.copy_tip_until {
                if Instant::now() < until {
                    ui.label(
                        egui::RichText::new("Copied!")
                            .color(egui::Color32::LIGHT_GREEN)
                            .small(),
                    );
                    ui.ctx().request_repaint_after(Duration::from_millis(100));
                } else {
                    self.copy_tip_until = None;
                }
            }
        });
    }

    #[cfg(feature = "events")]
    fn show_events_feature_tip(&mut self, _ui: &mut Ui) {}
}

impl App for DemoApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::SidePanel::right("right")
            .default_width(260.0)
            .show(ctx, |ui| {
                ScrollArea::vertical().show(ui, |ui| {
                    #[cfg(not(feature = "events"))]
                    self.show_events_feature_tip(ui);
                    if ui
                        .button("Reset Defaults")
                        .on_hover_text("Reset ALL settings, graph, layout & view state")
                        .clicked()
                    {
                        self.reset_all(ui);
                    }
                    CollapsingHeader::new("Graph / Layout")
                        .default_open(true)
                        .show(ui, |ui| self.ui_graph_section(ui));
                    self.ui_navigation(ui);
                    self.ui_layout_force_directed(ui);
                    self.ui_interaction(ui);
                    self.ui_style(ui);
                    self.ui_selected(ui);
                    self.ui_events(ui);
                    // debug section moved to overlay
                });
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            let settings_interaction = &egui_graphs::SettingsInteraction::new()
                .with_node_selection_enabled(self.settings_interaction.node_selection_enabled)
                .with_node_selection_multi_enabled(
                    self.settings_interaction.node_selection_multi_enabled,
                )
                .with_dragging_enabled(self.settings_interaction.dragging_enabled)
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
                .with_screen_padding(self.settings_navigation.screen_padding);
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

            let mut view = egui_graphs::GraphView::<
                _,
                _,
                _,
                _,
                _,
                _,
                LayoutStateForceDirected,
                LayoutForceDirected,
            >::new(&mut self.g)
            .with_interactions(settings_interaction)
            .with_navigations(settings_navigation)
            .with_styles(settings_style);
            #[cfg(feature = "events")]
            {
                view = view.with_events(&self.event_publisher);
            }
            ui.add(&mut view);
        });

        #[cfg(feature = "events")]
        self.handle_events();
        self.update_fps();
        self.overlay_debug(ctx);
    }
}

fn main() {
    run_native(
        "egui_graphs demo",
        eframe::NativeOptions::default(),
        Box::new(|cc| Ok(Box::new(DemoApp::new(cc)))),
    )
    .unwrap();
}
