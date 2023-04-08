use eframe::{run_native, App, CreationContext};
use egui::Context;
use egui_graph::Graph;
use petgraph::stable_graph::NodeIndex;
use rand::Rng;

const NODE_COUNT: usize = 50;
const EDGE_COUNT: usize = 100;

pub struct MyApp {
    graph: Graph<(), ()>,
}

impl MyApp {
    fn new(_: &CreationContext<'_>) -> Self {
        Self {
            graph: Graph::new(generate_random_graph(NODE_COUNT, EDGE_COUNT)),
        }
    }
}

impl App for MyApp {
    fn update(&mut self, ctx: &Context, _: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add(&mut self.graph);
        });
    }
}

fn generate_random_graph(node_count: usize, edge_count: usize) -> petgraph::Graph<(), ()> {
    let mut rng = rand::thread_rng();
    let mut graph = petgraph::Graph::<_, ()>::new();

    // Add nodes
    for _ in 0..node_count {
        graph.add_node(());
    }

    // Add random edges
    for _ in 0..edge_count {
        let source = NodeIndex::new(rng.gen_range(0..node_count));
        let target = NodeIndex::new(rng.gen_range(0..node_count));

        // Prevent self-loops
        if source != target {
            graph.add_edge(source, target, ());
        }
    }

    graph
}

fn main() {
    let native_options = eframe::NativeOptions::default();
    run_native(
        "egui-graph",
        native_options,
        Box::new(|cc| Box::new(MyApp::new(cc))),
    )
    .unwrap();
}
