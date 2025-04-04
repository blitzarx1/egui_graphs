use eframe::{run_native, App, CreationContext};
use egui::Context;
use egui_graphs::{Graph, GraphView, LayoutRandom, LayoutStateRandom};
use petgraph::{
    stable_graph::{StableGraph, StableUnGraph},
    Undirected,
};

pub struct UndirectedApp {
    g: Graph<(), (), Undirected>,
}

impl UndirectedApp {
    fn new(_: &CreationContext<'_>) -> Self {
        let g = generate_graph();
        Self { g: Graph::from(&g) }
    }
}

impl App for UndirectedApp {
    fn update(&mut self, ctx: &Context, _: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add(&mut GraphView::<
                _,
                _,
                _,
                _,
                _,
                _,
                LayoutStateRandom,
                LayoutRandom,
            >::new(&mut self.g));
        });
    }
}

fn generate_graph() -> StableGraph<(), (), Undirected> {
    let mut g = StableUnGraph::default();

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
        "egui_graphs_undirected_demo",
        native_options,
        Box::new(|cc| Ok(Box::new(UndirectedApp::new(cc)))),
    )
    .unwrap();
}
