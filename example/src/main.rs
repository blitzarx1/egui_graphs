use std::collections::hash_map::HashMap;

use eframe::{run_native, App, CreationContext};
use egui::{Context, Ui};
use egui_graph::Graph;
use petgraph::stable_graph::NodeIndex;
use rand::Rng;

const NODE_COUNT: usize = 50;
const EDGE_COUNT: usize = 100;
const MAX_RANK: usize = 6;

pub struct ExampleApp {
    graph: Graph<(), ()>,
    simulation_autofit: bool,
    simulation_drag: bool,
}

impl ExampleApp {
    fn new(_: &CreationContext<'_>) -> Self {
        Self {
            graph: Graph::new(generate_random_graph(NODE_COUNT, EDGE_COUNT), false, true),
            simulation_autofit: false,
            simulation_drag: true,
        }
    }

    fn handle_keys(&mut self, ui: &mut Ui) {
        ui.input(|i| {
            if i.key_pressed(egui::Key::Space) {
                self.graph.fit_screen();
            }
        });
    }
}

impl App for ExampleApp {
    fn update(&mut self, ctx: &Context, _: &mut eframe::Frame) {
        egui::SidePanel::right("right_panel")
            .default_width(300.)
            .show(ctx, |ui| {
                ui.label("* Use Left Mouse Button to pan the graph and drag nodes");
                ui.label("* Ctrl + Mouse Wheel to zoom");
                ui.label("* Press space to fit the graph to the screen");
                ui.separator();
                if ui.button("Randomize").clicked() {
                    self.graph = Graph::new(
                        generate_random_graph(NODE_COUNT, EDGE_COUNT),
                        self.simulation_autofit,
                        self.simulation_drag,
                    );
                }
                if ui
                    .checkbox(&mut self.simulation_autofit, "simulation autofit")
                    .clicked()
                {
                    self.graph.set_simulation_autofit(self.simulation_autofit);
                }
                if ui
                    .checkbox(&mut self.simulation_drag, "simulation drag")
                    .clicked()
                {
                    self.graph.set_simulation_drag(self.simulation_drag);
                }
            });
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add(&mut self.graph);
            self.handle_keys(ui);
        });
    }
}

fn generate_random_graph(node_count: usize, edge_count: usize) -> petgraph::Graph<(), ()> {
    let mut rng = rand::thread_rng();
    let mut graph = petgraph::Graph::<_, _>::new();
    let mut rank_map = HashMap::<usize, usize>::new();

    // Add nodes
    for _ in 0..node_count {
        graph.add_node(());
    }

    // Add random edges
    for _ in 0..edge_count {
        let mut edge_valid = false;
        let mut source = 0;
        let mut target = 0;
        while !edge_valid {
            source = rng.gen_range(0..node_count);
            target = rng.gen_range(0..node_count);

            let source_allowed = rank_map.get(&source).unwrap_or(&0) < &MAX_RANK;
            let target_allowed = rank_map.get(&target).unwrap_or(&0) < &MAX_RANK;

            edge_valid = source != target && source_allowed && target_allowed
        }

        graph.add_edge(NodeIndex::new(source), NodeIndex::new(target), ());
        *rank_map.entry(source).or_insert(0) += 1;
        *rank_map.entry(target).or_insert(0) += 1;
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
