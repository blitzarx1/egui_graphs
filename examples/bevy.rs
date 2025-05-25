use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPlugin};
use egui_graphs::{generate_simple_digraph, DefaultGraphView, Graph};

#[derive(Component)]
pub struct BevyGraph(pub Graph<(), ()>);

impl BevyGraph {
    fn new() -> Self {
        let g = generate_simple_digraph();
        Self(Graph::from(&g))
    }
}

fn setup(mut commands: Commands) {
    // add an entity with an egui_graphs::Graph component
    commands.spawn(BevyGraph::new());
}

fn update_graph(mut contexts: EguiContexts, mut q_graph: Query<&mut BevyGraph>) {
    let ctx = contexts.ctx_mut();
    let graph = q_graph.single_mut();

    egui::CentralPanel::default().show(ctx, |ui| {
        ui.add(&mut DefaultGraphView::new(&mut graph.unwrap().0));
    });
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(EguiPlugin {
            enable_multipass_for_primary_context: false,
        })
        .add_systems(Startup, setup)
        .add_systems(Update, update_graph)
        .run();
}
