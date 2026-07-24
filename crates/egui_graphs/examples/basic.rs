use eframe::{run_native, App, CreationContext, NativeOptions};
use egui_graphs::{DefaultGraphView, Graph};

pub struct BasicApp {
    g: Graph,
}

impl BasicApp {
    fn new(_: &CreationContext<'_>) -> Self {
        Self {
            g: generate_graph(),
        }
    }
}

impl App for BasicApp {
    fn ui(&mut self, ui: &mut egui::Ui, _: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ui, |ui| {
            DefaultGraphView::new().show(ui, &mut self.g);
        });
    }
}

fn generate_graph() -> Graph {
    let mut g = Graph::new();

    let a = g.add_node(());
    let b = g.add_node(());
    let c = g.add_node(());

    g.add_edge(a, b, ());
    g.add_edge(b, c, ());
    g.add_edge(c, a, ());

    g
}

fn main() {
    run_native(
        "basic",
        NativeOptions::default(),
        Box::new(|cc| Ok(Box::new(BasicApp::new(cc)))),
    )
    .unwrap();
}
