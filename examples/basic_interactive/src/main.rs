use eframe::{run_native, App, CreationContext};
use egui::Context;
use egui_graphs::{to_input_graph, Edge, GraphView, Node, SettingsInteraction};
use petgraph::stable_graph::StableGraph;

pub struct BasicInteractiveApp {
    g: StableGraph<Node<()>, Edge<()>>,
}

impl BasicInteractiveApp {
    fn new(_: &CreationContext<'_>) -> Self {
        let g = generate_graph();
        Self { g }
    }
}

impl App for BasicInteractiveApp {
    fn update(&mut self, ctx: &Context, _: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let interaction_settings = &SettingsInteraction::new()
                .with_dragging_enabled(true)
                .with_clicking_enabled(true)
                .with_folding_enabled(true)
                .with_selection_enabled(true)
                .with_selection_multi_enabled(true)
                .with_selection_depth(i32::MAX)
                .with_folding_depth(usize::MAX);
            ui.add(&mut GraphView::new(&mut self.g).with_interactions(interaction_settings));
        });
    }
}

fn generate_graph() -> StableGraph<Node<()>, Edge<()>> {
    let mut g: StableGraph<(), ()> = StableGraph::new();

    let a = g.add_node(());
    let b = g.add_node(());
    let c = g.add_node(());

    g.add_edge(a, b, ());
    g.add_edge(b, c, ());
    g.add_edge(c, a, ());

    to_input_graph(&g)
}

fn main() {
    let native_options = eframe::NativeOptions::default();
    run_native(
        "egui_graphs_basic_interactive_demo",
        native_options,
        Box::new(|cc| Box::new(BasicInteractiveApp::new(cc))),
    )
    .unwrap();
}
