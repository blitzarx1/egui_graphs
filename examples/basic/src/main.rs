use eframe::{run_native, App, CreationContext};
use egui::Context;
use egui_graphs::{Graph, GraphView};
use petgraph::stable_graph::StableGraph;

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
    fn update(&mut self, ctx: &Context, _: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            // TODO: make default layout implicit
            ui.add(&mut GraphView::<_, _, _, _, _, _, egui_graphs::layouts::Default>::new(&mut self.g));
        });
    }
}

fn generate_graph() -> Graph {
    let mut g = StableGraph::new();

    let a = g.add_node(());
    let b = g.add_node(());
    let c = g.add_node(());

    g.add_edge(a, b, ());
    g.add_edge(b, c, ());
    g.add_edge(c, a, ());

    Graph::from(&g)
}

fn main() {
    let native_options = eframe::NativeOptions::default();
    run_native(
        "egui_graphs_basic_demo",
        native_options,
        Box::new(|cc| Ok(Box::new(BasicApp::new(cc)))),
    )
    .unwrap();
}
