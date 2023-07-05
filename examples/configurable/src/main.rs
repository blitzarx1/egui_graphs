use std::sync::mpsc::{Receiver, Sender};
use std::time::Instant;

use eframe::{run_native, App, CreationContext};
use egui::plot::{Line, Plot, PlotPoints};
use egui::{CollapsingHeader, Color32, Context, ScrollArea, Slider, Ui, Vec2, Visuals};
use egui_graphs::{
    Change, Edge, GraphView, Node, SettingsInteraction, SettingsNavigation, SettingsStyle, to_input_graph,
};
use fdg_sim::glam::Vec3;
use fdg_sim::{ForceGraph, ForceGraphHelper, Simulation, SimulationParameters};
use petgraph::stable_graph::{EdgeIndex, NodeIndex, StableGraph};
use petgraph::visit::EdgeRef;
use petgraph::Directed;
use rand::Rng;
use settings::SettingsGraph;

mod settings;

const SIMULATION_DT: f32 = 0.035;
const FPS_LINE_COLOR: Color32 = Color32::from_rgb(128, 128, 128);
const CHANGES_LIMIT: usize = 100;

pub struct ConfigurableApp {
    g: StableGraph<Node<()>, Edge<()>>,
    sim: Simulation<(), f32>,

    settings_graph: SettingsGraph,
    settings_interaction: SettingsInteraction,
    settings_navigation: SettingsNavigation,
    settings_style: SettingsStyle,

    min_for_select: bool,
    max_for_select: bool,
    max_for_fold: bool,

    selected_nodes: Vec<Node<()>>,
    selected_edges: Vec<Edge<()>>,
    last_changes: Vec<Change>,

    simulation_stopped: bool,
    dark_mode: bool,

    fps: f64,
    fps_history: Vec<f64>,
    last_update_time: Instant,
    frames_last_time_span: usize,

    changes_receiver: Receiver<Change>,
    changes_sender: Sender<Change>,
}

impl ConfigurableApp {
    fn new(_: &CreationContext<'_>) -> Self {
        let settings_graph = SettingsGraph::default();
        let (g, sim) = generate(&settings_graph);
        let (changes_sender, changes_receiver) = std::sync::mpsc::channel();
        Self {
            g,
            sim,

            changes_receiver,
            changes_sender,

            settings_graph,

            settings_interaction: Default::default(),
            settings_navigation: Default::default(),
            settings_style: Default::default(),

            min_for_select: Default::default(),
            max_for_select: Default::default(),
            max_for_fold: Default::default(),

            selected_nodes: Default::default(),
            selected_edges: Default::default(),
            last_changes: Default::default(),

            simulation_stopped: false,
            dark_mode: true,

            fps: 0.,
            fps_history: Default::default(),
            last_update_time: Instant::now(),
            frames_last_time_span: 0,
        }
    }

    fn update_simulation(&mut self) {
        if self.simulation_stopped {
            return;
        }

        // the following manipulations is a hack to avoid having looped edges in the simulation
        // because they cause the simulation to blow up; this is the issue of the fdg_sim engine
        // we use for the simulation
        // * remove loop edges
        // * update simulation
        // * restore loop edges

        // remove looped edges
        let looped_nodes = {
            let graph = self.sim.get_graph_mut();
            let mut looped_nodes = vec![];
            let mut looped_edges = vec![];
            graph.edge_indices().for_each(|idx| {
                let edge = graph.edge_endpoints(idx).unwrap();
                let looped = edge.0 == edge.1;
                if looped {
                    looped_nodes.push((edge.0, ()));
                    looped_edges.push(idx);
                }
            });

            for idx in looped_edges {
                graph.remove_edge(idx);
            }

            self.sim.update(SIMULATION_DT);

                        looped_nodes
        };

        // restore looped edges
        let graph = self.sim.get_graph_mut();
        for (idx, _) in looped_nodes.iter() {
            graph.add_edge(*idx, *idx, 1.);
        }
    }

    /// Syncs the graph with the simulation.
    ///
    /// Changes location of nodes in `g` according to the locations in `sim`. If node from `g` is dragged its location is prioritized
    /// over the location of the corresponding node from `sim` and this location is set to the node from the `sim`.
    ///
    /// If node or edge is selected it is added to the corresponding selected field in `self`.
    fn sync_graph_with_simulation(&mut self) {
        self.selected_nodes = vec![];

        let g_indices = self.g.node_indices().collect::<Vec<_>>();
        g_indices.iter().for_each(|g_n_idx| {
            let g_n = self.g.node_weight_mut(*g_n_idx).unwrap();
            let sim_n = self.sim.get_graph_mut().node_weight_mut(*g_n_idx).unwrap();

            if g_n.dragged() {
                let loc = g_n.location();
                sim_n.location = Vec3::new(loc.x, loc.y, 0.);
                return;
            }

            let loc = sim_n.location;
            g_n.set_location(Vec2::new(loc.x, loc.y));

            if g_n.selected() {
                self.selected_nodes.push(g_n.clone());
            }

            // TODO: if node is folded make edge weight = 0.
            // if node is foldign root make edge weigh = num_folded_children (add num_folded_children to node fields)
        });
    }

    fn update_fps(&mut self) {
        self.frames_last_time_span += 1;
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_update_time);
        if elapsed.as_secs() >= 1 {
            self.last_update_time = now;
            self.fps = self.frames_last_time_span as f64 / elapsed.as_secs_f64();
            self.frames_last_time_span = 0;

            self.fps_history.push(self.fps);
            if self.fps_history.len() > 100 {
                self.fps_history.remove(0);
            }
        }
    }

    fn reset_graph(&mut self, ui: &mut Ui) {
        let settings_graph = SettingsGraph::default();
        let (g, sim) = generate(&settings_graph);

        self.g = g;
        self.sim = sim;
        self.settings_graph = settings_graph;
        self.last_changes = Default::default();

        GraphView::<(), (), Directed>::reset_metadata(ui);
    }

    fn handle_changes(&mut self) {
        self.changes_receiver.try_iter().for_each(|changes| {
            if self.last_changes.len() > CHANGES_LIMIT {
                self.last_changes.remove(0);
            }

            self.last_changes.push(changes);
        });
    }

    fn random_node_idx(&self) -> Option<NodeIndex> {
        let nodes_cnt = self.g.node_count();
        if nodes_cnt == 0 {
            return None;
        }

        let mut rng = rand::thread_rng();
        let random_n_idx = rng.gen_range(0..nodes_cnt);
        self.g.node_indices().nth(random_n_idx)
    }

    fn random_edge_idx(&self) -> Option<EdgeIndex> {
        let edges_cnt = self.g.edge_count();
        if edges_cnt == 0 {
            return None;
        }

        let mut rng = rand::thread_rng();
        let random_e_idx = rng.gen_range(0..edges_cnt);
        self.g.edge_indices().nth(random_e_idx)
    }

    fn remove_random_node(&mut self) {
        let idx = self.random_node_idx().unwrap();
        self.remove_node(idx);
    }

    fn add_random_node(&mut self) {
        let random_n_idx = self.random_node_idx();
        if random_n_idx.is_none() {
            return;
        }

        let random_n = self.g.node_weight(random_n_idx.unwrap()).unwrap();

        // location of new node is in surrounging of random existing node
        let mut rng = rand::thread_rng();
        let location = Vec2::new(
            random_n.location().x + 10. + rng.gen_range(0. ..50.),
            random_n.location().y + 10. + rng.gen_range(0. ..50.),
        );

        let idx = self.g.add_node(Node::new(location, ()));
        let n = self.g.node_weight_mut(idx).unwrap();
        *n = n.with_label(format!("{:?}", idx));
        let mut sim_node = fdg_sim::Node::new(idx.index().to_string().as_str(), ());
        sim_node.location = Vec3::new(location.x, location.y, 0.);
        self.sim.get_graph_mut().add_node(sim_node);
    }

    fn remove_node(&mut self, idx: NodeIndex) {
        // before removing nodes we need to remove all edges connected to it
        let neighbors = self.g.neighbors_undirected(idx).collect::<Vec<_>>();
        neighbors.iter().for_each(|n| {
            self.remove_edges(idx, *n);
            self.remove_edges(*n, idx);
        });

        self.g.remove_node(idx).unwrap();
        self.sim.get_graph_mut().remove_node(idx).unwrap();

        // update edges count
        self.settings_graph.count_edge = self.g.edge_count();
    }

    fn add_random_edge(&mut self) {
        let random_start = self.random_node_idx().unwrap();
        let random_end = self.random_node_idx().unwrap();

        self.add_edge(random_start, random_end);
    }

    fn add_edge(&mut self, start: NodeIndex, end: NodeIndex) {
        self.g.add_edge(start, end, Edge::new(()));
        self.sim.get_graph_mut().add_edge(start, end, 1.);
    }

    fn remove_random_edge(&mut self) {
        let random_e_idx = self.random_edge_idx();
        if random_e_idx.is_none() {
            return;
        }
        let endpoints = self.g.edge_endpoints(random_e_idx.unwrap()).unwrap();

        self.remove_edge(endpoints.0, endpoints.1);
    }

    /// Removes random edge. Can not remove edge by idx because
    /// there can be multiple edges between two nodes in 2 graphs
    /// and we can't be sure that they are indexed the same way.
    fn remove_edge(&mut self, start: NodeIndex, end: NodeIndex) {
        let g_idx = self.g.find_edge(start, end);
        if g_idx.is_none() {
            return;
        }

        self.g.remove_edge(g_idx.unwrap()).unwrap();

        let sim_idx = self.sim.get_graph_mut().find_edge(start, end).unwrap();
        self.sim.get_graph_mut().remove_edge(sim_idx).unwrap();
    }

    /// Removes all edges between two nodes
    fn remove_edges(&mut self, start: NodeIndex, end: NodeIndex) {
        let g_idxs = self
            .g
            .edges_connecting(start, end)
            .map(|e| e.id())
            .collect::<Vec<_>>();
        if g_idxs.is_empty() {
            return;
        }

        g_idxs.iter().for_each(|e| {
            self.g.remove_edge(*e).unwrap();
        });

        let sim_idxs = self
            .sim
            .get_graph()
            .edges_connecting(start, end)
            .map(|e| e.id())
            .collect::<Vec<_>>();

        sim_idxs.iter().for_each(|e| {
            self.sim.get_graph_mut().remove_edge(*e).unwrap();
        });
    }

    fn draw_section_client(&mut self, ui: &mut Ui) {
        CollapsingHeader::new("Client")
            .default_open(true)
            .show(ui, |ui| {
                ui.add_space(10.);

                ui.label("Simulation");
                ui.separator();

                ui.horizontal(|ui| {
                    if ui
                        .button(match self.simulation_stopped {
                            true => "start",
                            false => "stop",
                        })
                        .clicked()
                    {
                        self.simulation_stopped = !self.simulation_stopped;
                    };
                    if ui.button("reset").clicked() {
                        self.reset_graph(ui);
                    }
                });

                ui.add_space(10.);

                self.draw_counts_sliders(ui);

                ui.add_space(10.);

                ui.label("Style");
                ui.separator();

                self.draw_dark_mode(ui);
            });
    }

    fn draw_section_widget(&mut self, ui: &mut Ui) {
        CollapsingHeader::new("Widget")
        .default_open(true)
        .show(ui, |ui| {
            ui.add_space(10.);

            ui.label("SettingsNavigation");
            ui.separator();

            if ui
                .checkbox(&mut self.settings_navigation.fit_to_screen, "fit_to_screen")
                .changed()
                && self.settings_navigation.fit_to_screen
            {
                self.settings_navigation.zoom_and_pan = false
            };
            ui.label("Enable fit to screen to fit the graph to the screen on every frame.");

            ui.add_space(5.);

            ui.add_enabled_ui(!self.settings_navigation.fit_to_screen, |ui| {
                ui.vertical(|ui| {
                    ui.checkbox(&mut self.settings_navigation.zoom_and_pan, "zoom_and_pan");
                    ui.label("Zoom with ctrl + mouse wheel, pan with mouse drag.");
                }).response.on_disabled_hover_text("disable fit_to_screen to enable zoom_and_pan");
            });

            ui.add_space(10.);

            ui.label("SettingsStyle");
            ui.separator();

            ui.add(Slider::new(&mut self.settings_style.edge_radius_weight, 0. ..=5.)
            .text("edge_radius_weight"));
            ui.label("For every edge connected to node its radius is getting bigger by this value.");

            ui.add_space(5.);

            ui.checkbox(&mut self.settings_style.labels_always, "labels_always");
            ui.label("Wheter to show labels always or when interacted only.");

            ui.add_space(10.);

            ui.label("SettingsInteraction");
            ui.separator();

            ui.add_enabled_ui(!(self.settings_interaction.node_drag || self.settings_interaction.node_select || self.settings_interaction.node_multiselect || self.settings_interaction.node_fold), |ui| {
                ui.vertical(|ui| {
                    ui.checkbox(&mut self.settings_interaction.node_click, "node_click");
                    ui.label("Check click events in last changes");
                }).response.on_disabled_hover_text("node click is enabled when any of the interaction is also enabled");
            });

            ui.add_space(5.);

            if ui.checkbox(&mut self.settings_interaction.node_drag, "node_drag").clicked() && self.settings_interaction.node_drag {
                self.settings_interaction.node_click = true;
            };
            ui.label("To drag use LMB click + drag on a node.");

            ui.add_space(5.);

            ui.add_enabled_ui(!self.settings_interaction.node_multiselect, |ui| {
                ui.vertical(|ui| {
                    if ui.checkbox(&mut self.settings_interaction.node_select, "node_select").clicked() && self.settings_interaction.node_select {
                        self.settings_interaction.node_click = true;
                    };
                    ui.label("Enable select to select nodes with LMB click. If node is selected clicking on it again will deselect it.");
                }).response.on_disabled_hover_text("node_multiselect enables select");
            });

            if ui.checkbox(&mut self.settings_interaction.node_multiselect, "node_multiselect").changed() && self.settings_interaction.node_multiselect {
                self.settings_interaction.node_click = true;
                self.settings_interaction.node_select = true;
            }
            ui.label("Enable multiselect to select multiple nodes.");

            ui.add_enabled_ui(self.settings_interaction.node_select || self.settings_interaction.node_multiselect, |ui| {
                ui.horizontal(|ui| {
                    ui.add_enabled_ui(!self.min_for_select, |ui| {
                        if ui.checkbox(&mut self.max_for_select, "all_children").clicked() {
                            self.settings_interaction.selection_depth = i32::MAX;
                        }; 
                    });
                    ui.add_enabled_ui(!self.max_for_select, |ui| {
                        if ui.checkbox(&mut self.min_for_select, "all_parents").clicked() {
                            self.settings_interaction.selection_depth = i32::MIN;
                        };
                    });
                });
            });
            ui.add_enabled_ui((self.settings_interaction.node_select || self.settings_interaction.node_multiselect) && !(self.min_for_select || self.max_for_select), |ui| {
                ui.add(Slider::new(&mut self.settings_interaction.selection_depth, -10..=10)
                    .text("selection_depth"));
                ui.label("How deep into the neighbours of selected nodes should the selection go.");
            });

            ui.add_space(5.);

            if ui.checkbox(&mut self.settings_interaction.node_fold, "node_fold").clicked() && self.settings_interaction.node_fold {
                self.settings_interaction.node_click = true;
            };
            ui.label("To fold use LMB double click on a node. Currenly supports only one node folded at a time.");

            ui.add_enabled_ui(self.settings_interaction.node_fold, |ui| {
                ui.horizontal(|ui| {
                    if ui.checkbox(&mut self.max_for_fold, "all children").clicked() {
                        self.settings_interaction.folding_depth = usize::MAX;
                    };
                });
            });
            ui.add_enabled_ui(self.settings_interaction.node_fold && !self.max_for_fold, |ui| {
                ui.add(Slider::new(&mut self.settings_interaction.folding_depth, 0..=10)
                .text("folding_depth"));
                ui.label("How deep to fold childrens of the selected for folding node.");
            });

            ui.add_space(5.);

            ui.collapsing("selected", |ui| {
                ScrollArea::vertical().max_height(200.).show(ui, |ui| {
                    self.selected_nodes.iter().for_each(|node| {
                        ui.label(format!("{:?}", node));
                    });
                    self.selected_edges.iter().for_each(|edge| {
                        ui.label(format!("{:?}", edge));
                    });
                });
            });

            ui.collapsing("last changes", |ui| {
                ScrollArea::vertical().max_height(200.).show(ui, |ui| {
                    self.last_changes.iter().rev().for_each(|node| {
                        ui.label(format!("{:?}", node));
                    });
                });
            });
        });
    }

    fn draw_section_debug(&mut self, ui: &mut Ui) {
        CollapsingHeader::new("Debug")
            .default_open(false)
            .show(ui, |ui| {
                ui.add_space(10.);

                ui.vertical(|ui| {
                    ui.label(format!("fps: {:.1}", self.fps));
                    ui.add_space(10.);
                    self.draw_fps(ui);
                });
            });
    }

    fn draw_dark_mode(&mut self, ui: &mut Ui) {
        if self.dark_mode {
            ui.ctx().set_visuals(Visuals::dark())
        } else {
            ui.ctx().set_visuals(Visuals::light())
        }

        if ui
            .button({
                match self.dark_mode {
                    true => "🔆 light",
                    false => "🌙 dark",
                }
            })
            .clicked()
        {
            self.dark_mode = !self.dark_mode
        };
    }

    fn draw_fps(&self, ui: &mut Ui) {
        let points: PlotPoints = self
            .fps_history
            .iter()
            .enumerate()
            .map(|(i, val)| [i as f64, *val])
            .collect();

        let line = Line::new(points).color(FPS_LINE_COLOR);
        Plot::new("my_plot")
            .min_size(Vec2::new(100., 80.))
            .show_x(false)
            .show_y(false)
            .show_background(false)
            .show_axes([false, true])
            .allow_boxed_zoom(false)
            .allow_double_click_reset(false)
            .allow_drag(false)
            .allow_scroll(false)
            .allow_zoom(false)
            .show(ui, |plot_ui| plot_ui.line(line));
    }

    fn draw_counts_sliders(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            let before = self.settings_graph.count_node as i32;

            ui.add(Slider::new(&mut self.settings_graph.count_node, 1..=2500).text("nodes"));

            let delta = self.settings_graph.count_node as i32 - before;
            (0..delta.abs()).for_each(|_| {
                if delta > 0 {
                    self.add_random_node();
                    return;
                };
                self.remove_random_node();
            });
        });

        ui.horizontal(|ui| {
            let before = self.settings_graph.count_edge as i32;

            ui.add(Slider::new(&mut self.settings_graph.count_edge, 0..=5000).text("edges"));

            let delta = self.settings_graph.count_edge as i32 - before;
            (0..delta.abs()).for_each(|_| {
                if delta > 0 {
                    self.add_random_edge();
                    return;
                };
                self.remove_random_edge();
            });
        });
    }
}

impl App for ConfigurableApp {
    fn update(&mut self, ctx: &Context, _: &mut eframe::Frame) {
        egui::SidePanel::right("right_panel")
            .min_width(250.)
            .show(ctx, |ui| {
                ScrollArea::vertical().show(ui, |ui| {
                    self.draw_section_client(ui);

                    ui.add_space(10.);

                    self.draw_section_widget(ui);

                    ui.add_space(10.);

                    self.draw_section_debug(ui);
                });
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add(
                &mut GraphView::new(&mut self.g)
                    .with_interactions(&self.settings_interaction)
                    .with_navigations(&self.settings_navigation)
                    .with_styles(&self.settings_style)
                    .with_changes(&self.changes_sender),
            );
        });

        self.handle_changes();
        self.sync_graph_with_simulation();

        self.update_simulation();
        self.update_fps();
    }
}

fn generate(settings: &SettingsGraph) -> (StableGraph<Node<()>, Edge<()>>, Simulation<(), f32>) {
    let g = generate_random_graph(settings.count_node, settings.count_edge);
    let sim = construct_simulation(&g);

    (g, sim)
}

fn construct_simulation(g: &StableGraph<Node<()>, Edge<()>>) -> Simulation<(), f32> {
    // create force graph
    let mut force_graph = ForceGraph::with_capacity(g.node_count(), g.edge_count());
    g.node_indices().for_each(|idx| {
        let idx = idx.index();
        force_graph.add_force_node(format!("{}", idx).as_str(), ());
    });
    g.edge_indices().for_each(|idx| {
        let (source, target) = g.edge_endpoints(idx).unwrap();
        force_graph.add_edge(source, target, 1.);
    });

    // initialize simulation
    let mut params = SimulationParameters::default();
    let force = fdg_sim::force::fruchterman_reingold_weighted(100., 0.5);
    params.set_force(force);

    Simulation::from_graph(force_graph, params)
}

fn generate_random_graph(node_count: usize, edge_count: usize) -> StableGraph<Node<()>, Edge<()>> {
    let mut rng = rand::thread_rng();
    let mut graph = StableGraph::new();

    // add nodes
    for _ in 0..node_count {
        graph.add_node(());
    }

    // add random edges
    for _ in 0..edge_count {
        let source = rng.gen_range(0..node_count);
        let target = rng.gen_range(0..node_count);

        graph.add_edge(
            NodeIndex::new(source),
            NodeIndex::new(target),
            (),
        );
    }

    to_input_graph(&graph)
}

fn main() {
    let native_options = eframe::NativeOptions::default();
    run_native(
        "egui_graphs_configurable_demo",
        native_options,
        Box::new(|cc| Box::new(ConfigurableApp::new(cc))),
    )
    .unwrap();
}
