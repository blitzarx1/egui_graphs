use std::{collections::HashMap, time::Instant};

use eframe::{run_native, App, CreationContext};
use egui::{Context, Vec2};
use egui_graphs::{Edge, Elements, Graph, Node, Settings};
use fdg_sim::{ForceGraph, ForceGraphHelper, Simulation, SimulationParameters};
use petgraph::{stable_graph::NodeIndex, visit::IntoNodeReferences, Directed};
use rand::Rng;

const NODE_COUNT: usize = 300;
const EDGE_COUNT: usize = 400;
const SIMULATION_DT: f32 = 0.035;
const EDGE_SCALE_WEIGHT: f32 = 1.;

pub struct ExampleApp {
    simulation: Simulation<usize, String>,
    elements: Elements,
    settings: Settings,
    last_update_time: Instant,
    frames_last_time_span: usize,
    fps: f32,
}

impl ExampleApp {
    fn new(_: &CreationContext<'_>) -> Self {
        let settings = Settings::default();
        let (simulation, elements) = construct_simulation(NODE_COUNT, EDGE_COUNT);
        Self {
            simulation,
            settings,
            elements,
            last_update_time: Instant::now(),
            frames_last_time_span: 0,
            fps: 0.,
        }
    }

    fn update_simulation(&mut self) {
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

        self.simulation
            .get_graph()
            .node_references()
            .for_each(|(idx, node)| {
                let loc = node.location;
                self.elements.nodes.get_mut(&idx.index()).unwrap().location =
                    Vec2::new(loc.x, loc.y);
            });
    }

    // fn handle_keys(&mut self, ui: &mut Ui) {
    //     ui.input(|i| {
    //         if i.key_pressed(egui::Key::Space) {
    //             self.graph.fit_screen();
    //         }
    //     });
    // }

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
}

impl App for ExampleApp {
    fn update(&mut self, ctx: &Context, _: &mut eframe::Frame) {
        self.update_simulation();
        self.update_fps();

        egui::SidePanel::right("right_panel")
            .default_width(300.)
            .show(ctx, |ui| {
                ui.add_space(5.);
                if ui.button("randomize").clicked() {
                    let (simulation, elements) = construct_simulation(NODE_COUNT, EDGE_COUNT);
                    self.simulation = simulation;
                    self.elements = elements;

                    Graph::<usize, String, Directed>::reset_state(ui);
                }

                ui.add_space(10.);
                ui.label("Graph Settings");
                ui.separator();
                ui.horizontal(|ui| {
                    ui.checkbox(&mut self.settings.fit_to_screen, "autofit");
                    ui.add_enabled_ui(!self.settings.fit_to_screen, |ui| {
                        ui.checkbox(&mut self.settings.zoom_and_pan, "pan & zoom")
                            .on_disabled_hover_text("disabled autofit to enable pan & zoom");
                    });
                });
                ui.add_space(5.);
                ui.checkbox(&mut self.settings.node_drag, "drag nodes");
                egui::TopBottomPanel::bottom("bottom_panel").show_inside(ui, |ui| {
                    ui.add_space(5.);
                    ui.label(format!("fps: {:.1}", self.fps));
                });
            });
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add(Graph::new(
                self.simulation.get_graph_mut(),
                &mut self.elements,
                &self.settings,
            ));
            // self.handle_keys(ui);
        });
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
        let mut pair = [source.index(), target.index()];
        // keep sorted pairs of nodes to distinguish between directed edges
        pair.sort();
        let key = (pair[0], pair[1]);
        edges.entry(key).or_insert_with(Vec::new);
        edges
            .get_mut(&key)
            .unwrap()
            .push(Edge::new(source.index(), target.index()));

        nodes.get_mut(&source.index()).unwrap().radius += EDGE_SCALE_WEIGHT;
        nodes.get_mut(&target.index()).unwrap().radius += EDGE_SCALE_WEIGHT;
    });
    let elements = Elements { nodes, edges };

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
