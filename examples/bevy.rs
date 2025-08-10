use bevy::prelude::*;
use bevy_egui::EguiPrimaryContextPass;
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

fn setup_graph(mut commands: Commands) {
    commands.spawn(BevyGraph::new());
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn update_graph(mut contexts: EguiContexts, mut q_graph: Query<&mut BevyGraph>) {
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };
    if let Ok(mut bevy_graph) = q_graph.single_mut() {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add(&mut DefaultGraphView::new(&mut bevy_graph.0));
        });
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(EguiPlugin::default())
        .add_systems(Startup, (setup_graph, setup_camera))
        .add_systems(EguiPrimaryContextPass, update_graph)
        .run();
}
