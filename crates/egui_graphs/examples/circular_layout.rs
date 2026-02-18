use eframe::egui;
use egui_graphs::{
    DefaultEdgeShape, DefaultNodeShape, Graph, GraphView, LayoutCircular, LayoutStateCircular,
    SettingsInteraction, SettingsStyle,
};
use petgraph::{
    stable_graph::{DefaultIx, StableGraph},
    Directed,
};

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "Circular Layout Example",
        native_options,
        Box::new(|_cc| Ok::<Box<dyn eframe::App>, _>(Box::new(CircularExample::new()))),
    )
}

struct CircularExample {
    g: Graph<(), ()>,
}

impl CircularExample {
    fn new() -> Self {
        let mut graph = StableGraph::new();

        // Create some nodes
        let nodes: Vec<_> = (0..8).map(|_| graph.add_node(())).collect();

        // Add some edges in a ring
        for i in 0..nodes.len() {
            graph.add_edge(nodes[i], nodes[(i + 1) % nodes.len()], ());
        }

        let mut g = Graph::from(&graph);

        // Set labels
        for (i, idx) in nodes.iter().enumerate() {
            if let Some(node) = g.node_mut(*idx) {
                node.set_label(format!("Node {}", i));
            }
        }

        Self { g }
    }
}

impl eframe::App for CircularExample {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Circular Layout Example");
            ui.label("8 nodes arranged in a circle");

            ui.add(
                &mut GraphView::<
                    (),
                    (),
                    Directed,
                    DefaultIx,
                    DefaultNodeShape,
                    DefaultEdgeShape,
                    LayoutStateCircular,
                    LayoutCircular,
                >::new(&mut self.g)
                .with_interactions(&SettingsInteraction::new())
                .with_styles(&SettingsStyle::new()),
            );
        });
    }
}
