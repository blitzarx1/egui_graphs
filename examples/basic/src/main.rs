use std::collections::HashMap;

use eframe::{run_native, App, CreationContext};
use egui::Context;
use egui_graphs::{Edge, Elements, GraphView, Node};

const SIDE_SIZE: f32 = 50.;

pub struct BasicApp {
    elements: Elements,
}

impl BasicApp {
    fn new(_: &CreationContext<'_>) -> Self {
        let elements = generate_graph();
        Self { elements }
    }
}

impl App for BasicApp {
    fn update(&mut self, ctx: &Context, _: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add(GraphView::new(&self.elements));
        });
    }
}

fn generate_graph() -> Elements {
    let mut nodes = HashMap::new();
    nodes.insert(0, Node::new(0, egui::Vec2::new(0., SIDE_SIZE)));
    nodes.insert(1, Node::new(1, egui::Vec2::new(-SIDE_SIZE, 0.)));
    nodes.insert(2, Node::new(2, egui::Vec2::new(SIDE_SIZE, 0.)));

    let mut edges = HashMap::new();
    edges.insert((0, 1), vec![Edge::new(0, 1, 0)]);
    edges.insert((1, 2), vec![Edge::new(1, 2, 0)]);
    edges.insert((2, 0), vec![Edge::new(2, 0, 0)]);

    Elements::new(nodes, edges)
}

fn main() {
    let native_options = eframe::NativeOptions::default();
    run_native(
        "egui_graphs_basic_demo",
        native_options,
        Box::new(|cc| Box::new(BasicApp::new(cc))),
    )
    .unwrap();
}
