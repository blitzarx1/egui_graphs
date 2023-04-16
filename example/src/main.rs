use std::time::Instant;

use eframe::{run_native, App, CreationContext};
use egui::{Context, Ui};
use egui_graphs::{Graph, Settings};
use petgraph::stable_graph::NodeIndex;
use rand::Rng;

const NODE_COUNT: usize = 300;
const EDGE_COUNT: usize = 400;

pub struct ExampleApp {
    graph: Graph<(), ()>,
    settings: Settings,
    last_update_time: Instant,
    frames_last_time_span: usize,
    fps: f32,
}

impl ExampleApp {
    fn new(_: &CreationContext<'_>) -> Self {
        let settings = Settings::default();
        Self {
            graph: Graph::new(
                generate_random_graph(NODE_COUNT, EDGE_COUNT),
                settings.clone(),
            ),
            settings,
            last_update_time: Instant::now(),
            frames_last_time_span: 0,
            fps: 0.,
        }
    }

    fn handle_keys(&mut self, ui: &mut Ui) {
        ui.input(|i| {
            if i.key_pressed(egui::Key::Space) {
                self.graph.fit_screen();
            }
        });
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
}

impl App for ExampleApp {
    fn update(&mut self, ctx: &Context, _: &mut eframe::Frame) {
        self.update_fps();

        egui::SidePanel::right("right_panel")
            .default_width(300.)
            .show(ctx, |ui| {
                ui.label("* Use Left Mouse Button to pan the graph and drag nodes");
                ui.label("* Ctrl + Mouse Wheel to zoom");
                ui.label("* Press Space to fit the graph to the screen");

                ui.separator();
                if ui
                    .checkbox(&mut self.settings.autofit, "autofit")
                    .clicked()
                {
                    self.graph
                        .set_autofit(self.settings.autofit);
                }
                ui.label("autofit disables all other interactions with the graph and fits the graph to the screen on every simulation fram update");
                ui.add_space(10.);

                ui.add_enabled_ui(!self.settings.autofit, |ui| {
                    if ui
                        .checkbox(&mut self.settings.simulation_drag, "simulation drag")
                        .clicked()
                    {
                        self.graph
                            .set_simulation_drag(self.settings.simulation_drag);
                    }
                    ui.label("simulation drag starts the simulation when a node is dragged");
                });
                ui.add_space(10.);

                if ui.button("Randomize").clicked() {
                    self.graph = Graph::new(
                        generate_random_graph(NODE_COUNT, EDGE_COUNT),
                        self.settings.clone(),
                    );
                }
                egui::TopBottomPanel::bottom("bottom_panel").show_inside( ui, |ui| {
                    ui.label(format!("fps: {:.1}", self.fps));
                });
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

    // Add nodes
    for _ in 0..node_count {
        graph.add_node(());
    }

    // Add random edges
    for _ in 0..edge_count {
        let source = rng.gen_range(0..node_count);
        let target = rng.gen_range(0..node_count);

        graph.add_edge(NodeIndex::new(source), NodeIndex::new(target), ());
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
