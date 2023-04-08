use eframe::{run_native, App, CreationContext};
use egui::Context;
use egui_graph::Graph;

pub struct MyApp {
    graph: Graph<(), ()>,
}

impl MyApp {
    fn new(_: &CreationContext<'_>) -> Self {
        let mut g = petgraph::Graph::<_, ()>::new();
        let a = g.add_node(());
        let b = g.add_node(());
        g.add_edge(a, b, ());
        Self {
            graph: Graph::new(g),
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

fn main() {
    let native_options = eframe::NativeOptions::default();
    run_native(
        "egui-graph",
        native_options,
        Box::new(|cc| Box::new(MyApp::new(cc))),
    )
    .unwrap();
}
