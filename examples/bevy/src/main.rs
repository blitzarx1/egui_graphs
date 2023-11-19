use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPlugin};
use eframe::{run_native, App, CreationContext};
use egui::Context;
use egui_graphs::{DefaultEdgeShape, DefaultNodeShape, Graph, GraphView};
use petgraph::stable_graph::StableGraph;

fn main() {
    // let native_options = eframe::NativeOptions::default();
    // run_native(
    //     "egui_graphs_basic_demo",
    //     native_options,
    //     Box::new(|cc| Box::new(BasicApp::new(cc))),
    // )
    // .unwrap();

    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(EguiPlugin)
        // Systems that create Egui widgets should be run during the `CoreSet::Update` set,
        // or after the `EguiSet::BeginFrame` system (which belongs to the `CoreSet::PreUpdate` set).
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
    commands.spawn(BasicGraph::new());
}

fn update_graph(mut contexts: EguiContexts, mut q_graph: Query<&mut BasicGraph>) {
    let ctx = contexts.ctx_mut();
    let mut graph = q_graph.single_mut().unwrap();

    egui::CentralPanel::default().show(ctx, |ui| {
        ui.add(&mut GraphView::<
            _,
            _,
            _,
            _,
            DefaultNodeShape,
            DefaultEdgeShape<_>,
        >::new(&mut graph.0));
    });
}
