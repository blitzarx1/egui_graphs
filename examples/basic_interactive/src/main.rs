use eframe::{run_native, App, CreationContext};
use egui::Context;
use egui_graphs::{Edge, GraphView, Node, SettingsInteraction};
use petgraph::stable_graph::StableGraph;

const SIDE_SIZE: f32 = 50.;

pub struct BasicInteractiveApp {
    g: StableGraph<Node<()>, Edge<()>>,
}

impl BasicInteractiveApp {
    fn new(_: &CreationContext<'_>) -> Self {
        let g = generate_graph();
        Self { g }
    }
}

impl App for BasicInteractiveApp {
    fn update(&mut self, ctx: &Context, _: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add(
                &mut GraphView::new(&mut self.g).with_interactions(&SettingsInteraction {
                    node_drag: true,
                    node_click: true,
                    node_select: true,
                    node_multiselect: true,
                    ..Default::default()
                }),
            );
        });
    }
}

fn generate_graph() -> StableGraph<Node<()>, Edge<()>> {
    let mut g: StableGraph<Node<()>, Edge<()>> = StableGraph::new();

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
        "egui_graphs_basic_interactive_demo",
        native_options,
        Box::new(|cc| Box::new(BasicInteractiveApp::new(cc))),
    )
    .unwrap();
}
