use eframe::{run_native, App, CreationContext};
use egui::{Context, Window};
use egui_graphs::{DefaultGraphView, Graph};
use petgraph::stable_graph::StableGraph;

pub struct WindowApp {
    g: Graph,
}

impl WindowApp {
    fn new(_: &CreationContext<'_>) -> Self {
        let g = generate_graph();
        Self { g: Graph::from(&g) }
    }
}

impl App for WindowApp {
    fn update(&mut self, ctx: &Context, _: &mut eframe::Frame) {
        Window::new("graph").show(ctx, |ui| {
            ui.add(&mut DefaultGraphView::new(&mut self.g));
        });
    }
}

fn generate_graph() -> StableGraph<(), ()> {
    let mut g = StableGraph::new();

    let a = g.add_node(());
    let b = g.add_node(());
    let c = g.add_node(());

    g.add_edge(a, b, ());
    g.add_edge(b, c, ());
    g.add_edge(c, a, ());

    g
}

fn main() {
    let native_options = eframe::NativeOptions::default();
    run_native(
        "egui_graphs_window_demo",
        native_options,
        Box::new(|cc| Ok(Box::new(WindowApp::new(cc)))),
    )
    .unwrap();
}
