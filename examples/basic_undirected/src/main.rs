use eframe::{run_native, App, CreationContext};
use egui::Context;
use egui_graphs::{Edge, Graph, GraphView, Node};
use petgraph::{stable_graph::StableUnGraph, Undirected};

const SIDE_SIZE: f32 = 50.;

pub struct BasicUndirectedApp {
    g: Graph<(), (), Undirected>,
}

impl BasicUndirectedApp {
    fn new(_: &CreationContext<'_>) -> Self {
        let g = generate_graph();
        Self { g }
    }
}

impl App for BasicUndirectedApp {
    fn update(&mut self, ctx: &Context, _: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add(&mut GraphView::new(&mut self.g));
        });
    }
}

fn generate_graph() -> Graph<(), (), Undirected> {
    let mut g: Graph<(), (), Undirected> = StableUnGraph::default();

    let a = g.add_node(Node::new(egui::Vec2::new(0., SIDE_SIZE), ()));
    let b = g.add_node(Node::new(egui::Vec2::new(-SIDE_SIZE, 0.), ()));
    let c = g.add_node(Node::new(egui::Vec2::new(SIDE_SIZE, 0.), ()));

    g.add_edge(a, b, Edge::new(()));
    g.add_edge(b, c, Edge::new(()));
    g.add_edge(c, a, Edge::new(()));

    g
}

fn main() {
    let native_options = eframe::NativeOptions::default();
    run_native(
        "egui_graphs_basic_undirected_demo",
        native_options,
        Box::new(|cc| Box::new(BasicUndirectedApp::new(cc))),
    )
    .unwrap();
}
