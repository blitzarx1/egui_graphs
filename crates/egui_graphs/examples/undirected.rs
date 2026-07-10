use eframe::{run_native, App, CreationContext};
use egui_graphs::{generate_simple_ungraph, DefaultGraphView, Graph};
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
    fn ui(&mut self, ui: &mut egui::Ui, _: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ui, |ui| {
            DefaultGraphView::new().show(ui, &mut self.g);
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
