use std::{collections::HashMap, time::Instant};

use eframe::{run_native, App, CreationContext};
use egui::plot::{Line, Plot, PlotPoints};
use egui::{CollapsingHeader, Color32, Context, ScrollArea, Vec2};
use egui_graphs::{Changes, Edge, Elements, GraphView, Node, Settings};
use fdg_sim::glam::Vec3;
use fdg_sim::{ForceGraph, ForceGraphHelper, Simulation, SimulationParameters};
use petgraph::stable_graph::NodeIndex;
use petgraph::visit::IntoNodeReferences;
use rand::Rng;

const NODE_COUNT: usize = 3;
const EDGE_COUNT: usize = 5;
const SIMULATION_DT: f32 = 0.035;
const EDGE_SCALE_WEIGHT: f32 = 1.;
const FPS_LINE_COLOR: Color32 = Color32::from_rgb(255, 255, 255);

pub struct ExampleApp {
    simulation: Simulation<usize, String>,
    elements: Elements,
    settings: Settings,

    selected_nodes: Vec<Node>,
    selected_edges: Vec<Edge>,

    simulation_stopped: bool,

    fps: f64,
    fps_history: Vec<f64>,
    last_update_time: Instant,
    frames_last_time_span: usize,
}

impl ExampleApp {
    fn new(_: &CreationContext<'_>) -> Self {
        let settings = Settings::default();
        let (simulation, elements) = construct_simulation(NODE_COUNT, EDGE_COUNT);
        Self {
            simulation,
            settings,
            elements,

            selected_nodes: Default::default(),
            selected_edges: Default::default(),

            simulation_stopped: false,

            fps: 0.,
            fps_history: Default::default(),
            last_update_time: Instant::now(),
            frames_last_time_span: 0,
        }
    }

    /// sync elements with simulation and state
    fn sync(&mut self) {
        self.selected_nodes = Default::default();
        self.selected_edges = Default::default();

        self.simulation
            .get_graph()
            .node_references()
            .for_each(|(idx, sim_node)| {
                let el_node: &mut Node = self.elements.get_node_mut(&idx.index()).unwrap();

                // sync location only if it was not dragged
                if Vec3::new(el_node.location.x, el_node.location.y, 0.) == sim_node.old_location {
                    el_node.location = Vec2::new(sim_node.location.x, sim_node.location.y);
                };

                if el_node.selected {
                    self.selected_nodes.push(el_node.clone());
                };
            });

        self.elements.get_edges().iter().for_each(|(_, edges)| {
            edges.iter().for_each(|e| {
                if e.selected {
                    self.selected_edges.push(e.clone());
                }
            });
        });
    }

    fn update_simulation(&mut self) {
        if self.simulation_stopped {
            return;
        }
        let looped_nodes = {
            // remove looped edges
            let graph = self.simulation.get_graph_mut();
            let mut looped_nodes = vec![];
            let mut looped_edges = vec![];
            graph.edge_indices().for_each(|idx| {
                let edge = graph.edge_endpoints(idx).unwrap();
                let looped = edge.0 == edge.1;
                if looped {
                    let edge_weight = graph.edge_weight(idx).unwrap().clone();
                    looped_nodes.push((edge.0, edge_weight));
                    looped_edges.push(idx);
                }
            });

            for idx in looped_edges {
                graph.remove_edge(idx);
            }

            self.simulation.update(SIMULATION_DT);

            looped_nodes
        };

        // restore looped edges
        let graph = self.simulation.get_graph_mut();
        for (idx, w) in looped_nodes.iter() {
            graph.add_edge(*idx, *idx, w.clone());
        }
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

    fn apply_changes(&mut self, changes: Changes) {
        if !changes.is_some() {
            return;
        }

        changes.nodes.iter().for_each(|(idx, change)| {
            if let Some(location_change) = change.location {
                let sim_node = self
                    .simulation
                    .get_graph_mut()
                    .node_weight_mut(NodeIndex::new(*idx))
                    .unwrap();
                sim_node.location = Vec3::new(location_change.x, location_change.y, 0.);
                sim_node.velocity = sim_node.location - sim_node.old_location;

                let el_node = self.elements.get_node_mut(idx).unwrap();
                el_node.location = location_change;
            }
            if let Some(radius_change) = change.radius {
                let node = self.elements.get_node_mut(idx).unwrap();
                node.radius = radius_change;
            }
            if let Some(color_change) = change.color {
                let node = self.elements.get_node_mut(idx).unwrap();
                node.color = color_change;
            }
            if let Some(dragged_change) = change.dragged {
                let node = self.elements.get_node_mut(idx).unwrap();
                node.dragged = dragged_change;
            }
            if let Some(selected_change) = change.selected {
                let node = self.elements.get_node_mut(idx).unwrap();
                node.selected = selected_change;

                // select all connected nodes and edges
                self.simulation
                    .get_graph()
                    .neighbors(NodeIndex::new(*idx))
                    .for_each(|neighbour| {
                        let node = self.elements.get_node_mut(&neighbour.index()).unwrap();
                        node.selected = selected_change;

                        if let Some(edges_from) =
                            self.elements.get_edges_between_mut(idx, &neighbour.index())
                        {
                            edges_from.iter_mut().for_each(|edge| {
                                edge.selected = selected_change;
                            });
                        }

                        if let Some(edges_to) =
                            self.elements.get_edges_between_mut(&neighbour.index(), idx)
                        {
                            edges_to.iter_mut().for_each(|edge| {
                                edge.selected = selected_change;
                            });
                        }
                    });
            }
        });

        changes.edges.iter().for_each(|(idx, change)| {
            if let Some(width_change) = change.width {
                let edge = self.elements.get_edge_mut(idx).unwrap();
                edge.width = width_change;
            }
            if let Some(curve_size_change) = change.curve_size {
                let edge = self.elements.get_edge_mut(idx).unwrap();
                edge.curve_size = curve_size_change;
            }
            if let Some(tip_size_change) = change.tip_size {
                let edge = self.elements.get_edge_mut(idx).unwrap();
                edge.tip_size = tip_size_change;
            }
            if let Some(selected_change) = change.selected {
                let edge = self.elements.get_edge_mut(idx).unwrap();
                edge.selected = selected_change;
            }
        });
    }
}

impl App for ExampleApp {
    fn update(&mut self, ctx: &Context, _: &mut eframe::Frame) {
        self.update_simulation();
        self.sync();
        self.update_fps();

        egui::SidePanel::right("right_panel")
            .default_width(300.)
            .show(ctx, |ui| {
                ScrollArea::vertical().show(ui, |ui|{
                    CollapsingHeader::new("WIDGET SETTINGS")
                    .default_open(true)
                    .show(ui, |ui| {
                        ui.add_space(10.);

                        ui.label("Navigation");
                        ui.separator();

                        if ui
                            .checkbox(&mut self.settings.fit_to_screen, "autofit")
                            .changed()
                            && self.settings.fit_to_screen
                        {
                            self.settings.zoom_and_pan = false
                        };
                        ui.label("Enable autofit to fit the graph to the screen on every frame.");

                        ui.add_space(5.);

                        ui.add_enabled_ui(!self.settings.fit_to_screen, |ui| {
                            ui.checkbox(&mut self.settings.zoom_and_pan, "pan & zoom")
                                .on_disabled_hover_text("disabled autofit to enable pan & zoom");
                            ui.label("Enable pan and zoom. To pan use LMB + drag and to zoom use Ctrl + Mouse Wheel.");
                        });

                        ui.add_space(10.);

                        ui.label("Interactions");
                        ui.separator();

                        ui.checkbox(&mut self.settings.node_drag, "drag");
                        ui.label("Enable drag. To drag use LMB + drag on a node.");

                        ui.add_space(5.);

                        ui.checkbox(&mut self.settings.node_select, "select");
                        ui.label("Enable select to select nodes with LMB click. If node is selected clicking on it again will deselect it.");

                        ui.add_space(5.);

                        ui.add_enabled_ui(self.settings.node_select, |ui| {
                            ui.checkbox(&mut self.settings.node_multiselect, "multiselect")
                                .on_disabled_hover_text("enable select to enable multiselect");
                            ui.label("Enable multiselect to select multiple nodes.");
                        });

                        ui.add_space(5.);

                        ui.collapsing("Selected", |ui| {
                            ScrollArea::vertical().max_height(200.).show(ui, |ui| {
                                self.selected_nodes.iter().for_each(|node| {
                                    ui.label(format!("{:?}", node));
                                });
                                self.selected_edges.iter().for_each(|edge| {
                                    ui.label(format!("{:?}", edge));
                                });
                            });
                        });
                    });


                ui.add_space(20.);

                CollapsingHeader::new("APP SETTINGS")
                    .default_open(true)
                    .show(ui, |ui| {
                        ui.add_space(10.);

                        ui.label("Simulation");
                        ui.separator();

                        if ui.button("randomize").clicked() {
                            let (simulation, elements) =
                                construct_simulation(NODE_COUNT, EDGE_COUNT);
                            self.simulation = simulation;
                            self.elements = elements;

                            GraphView::reset_metadata(ui);
                        }
                        ui.label("Randomize the graph.");

                        ui.add_space(5.);

                        ui.checkbox(&mut self.simulation_stopped, "stop");
                        ui.label("Stop the simulation.");
                    });

                egui::TopBottomPanel::bottom("bottom_panel").show_inside(ui, |ui| {
                    ui.add_space(5.);
                    ui.label(format!("fps: {:.1}", self.fps));

                    let sin: PlotPoints = self
                        .fps_history
                        .iter()
                        .enumerate()
                        .map(|(i, val)| [i as f64, *val])
                        .collect();
                    let line = Line::new(sin).color(FPS_LINE_COLOR);
                    Plot::new("my_plot")
                        .height(100.)
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
                });
            });
        });
        let widget = &GraphView::new(&self.elements, &self.settings);
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add(widget);
        });

        self.apply_changes(widget.last_changes());
    }
}

fn construct_simulation(
    node_count: usize,
    edge_count: usize,
) -> (Simulation<usize, String>, Elements) {
    // create graph
    let graph = generate_random_graph(node_count, edge_count);

    // init simulation
    let mut force_graph = ForceGraph::with_capacity(node_count, edge_count);
    graph.node_indices().for_each(|idx| {
        let idx = idx.index();
        force_graph.add_force_node(format!("{}", idx).as_str(), idx);
    });
    graph.edge_indices().for_each(|idx| {
        let (source, target) = graph.edge_endpoints(idx).unwrap();
        force_graph.add_edge(source, target, graph.edge_weight(idx).unwrap().clone());
    });
    let simulation = Simulation::from_graph(force_graph, SimulationParameters::default());

    // collect elements
    let mut nodes = HashMap::with_capacity(node_count);
    let mut edges = HashMap::with_capacity(edge_count);
    simulation.get_graph().node_indices().for_each(|idx| {
        let loc = simulation.get_graph().node_weight(idx).unwrap().location;
        nodes.insert(idx.index(), Node::new(Vec2::new(loc.x, loc.y)));
    });
    simulation.get_graph().edge_indices().for_each(|idx| {
        let (source, target) = simulation.get_graph().edge_endpoints(idx).unwrap();

        let key = (source.index(), target.index());
        edges.entry(key).or_insert_with(Vec::new);

        let edges_list = edges.get_mut(&key).unwrap();
        let list_idx = edges_list.len();

        edges_list.push(Edge::new(source.index(), target.index(), list_idx));

        nodes.get_mut(&source.index()).unwrap().radius += EDGE_SCALE_WEIGHT;
        nodes.get_mut(&target.index()).unwrap().radius += EDGE_SCALE_WEIGHT;
    });
    let elements = Elements::new(nodes, edges);

    (simulation, elements)
}

fn generate_random_graph(node_count: usize, edge_count: usize) -> petgraph::Graph<usize, String> {
    let mut rng = rand::thread_rng();
    let mut graph = petgraph::Graph::new();

    // add nodes
    for i in 0..node_count {
        graph.add_node(i);
    }

    // add random edges
    for _ in 0..edge_count {
        let source = rng.gen_range(0..node_count);
        let target = rng.gen_range(0..node_count);

        graph.add_edge(
            NodeIndex::new(source),
            NodeIndex::new(target),
            format!("{} -> {}", source, target),
        );
    }

    graph
}

fn main() {
    let native_options = eframe::NativeOptions::default();
    run_native(
        "egui-graph",
        native_options,
        Box::new(|cc| Box::new(ExampleApp::new(cc))),
    )
    .unwrap();
}
