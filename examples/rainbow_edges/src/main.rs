use edge::RainbowEdgeShape;
use eframe::{run_native, App, CreationContext};
use egui::Context;
use egui_graphs::{DefaultNodeShape, Graph, GraphView};
use petgraph::{csr::DefaultIx, stable_graph::StableGraph, Directed};

mod edge;

pub struct RainbowEdgesApp {
    g: Graph<(), (), Directed, DefaultIx, DefaultNodeShape, RainbowEdgeShape>,
}

impl RainbowEdgesApp {
    fn new(_: &CreationContext<'_>) -> Self {
        let g = generate_graph();
        Self { g: Graph::from(&g) }
    }
}

impl App for RainbowEdgesApp {
    fn update(&mut self, ctx: &Context, _: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add(
                &mut GraphView::<_, _, _, _, DefaultNodeShape, RainbowEdgeShape>::new(&mut self.g)
                    .with_interactions(
                        &egui_graphs::SettingsInteraction::default().with_dragging_enabled(true),
                    ),
            );
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
        "egui_graphs_rainbow_edges_demo",
        native_options,
        Box::new(|cc| Ok(Box::new(RainbowEdgesApp::new(cc)))),
    )
    .unwrap();
}
