use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPlugin};
use egui_graphs::{Graph, GraphView};
use petgraph::stable_graph::StableGraph;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(EguiPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, update_graph)
        .run();
}

#[derive(Component)]
pub struct BasicGraph(pub Graph<(), ()>);

impl BasicGraph {
    fn new() -> Self {
        let g = generate_graph();
        Self(Graph::from(&g))
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

fn setup(mut commands: Commands) {
    // add an entity with an egui_graphs::Graph component
    commands.spawn(BasicGraph::new());
}

fn update_graph(mut contexts: EguiContexts, mut q_graph: Query<&mut BasicGraph>) {
    let ctx = contexts.ctx_mut();
    let mut graph = q_graph.single_mut();

    egui::CentralPanel::default().show(ctx, |ui| {
        ui.add(&mut GraphView::new(&mut graph.0));
    });
}
