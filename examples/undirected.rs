use eframe::{run_native, App, CreationContext};
use egui::Context;
use egui_graphs::{generate_simple_ungraph, Graph, GraphView};
use petgraph::Undirected;

pub struct UndirectedApp {
    g: Graph<(), (), Undirected>,
}

impl UndirectedApp {
    fn new(_: &CreationContext<'_>) -> Self {
        let g = generate_simple_ungraph();
        Self { g: Graph::from(&g) }
    }
}

impl App for UndirectedApp {
    fn update(&mut self, ctx: &Context, _: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add(&mut GraphView::<_, _, _>::new(&mut self.g));
        });
    }
}

fn main() {
    let native_options = eframe::NativeOptions::default();
    run_native(
        "undirected",
        native_options,
        Box::new(|cc| Ok(Box::new(UndirectedApp::new(cc)))),
    )
    .unwrap();
}
