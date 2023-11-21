use bevy::prelude::*;
use bevy_egui::{
    egui::{self, Layout},
    EguiContexts, EguiPlugin,
};
use egui_graphs::{DefaultEdgeShape, DefaultNodeShape, Graph, GraphView, SettingsNavigation};
use petgraph::stable_graph::StableGraph;

#[derive(Resource)]
struct GraphProvider {
    g: Graph<(), ()>,
}

impl Default for GraphProvider {
    fn default() -> Self {
        let g = generate_graph();
        Self { g: Graph::from(&g) }
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

#[derive(Resource, Deref, DerefMut)]
struct OriginalCameraTransform(Transform);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(EguiPlugin)
        .init_resource::<GraphProvider>()
        .add_systems(Update, ui_example_system)
        .run();
}

fn ui_example_system(mut contexts: EguiContexts, mut graph_provider: ResMut<GraphProvider>) {
    let ctx = contexts.ctx_mut();
    let available_width = ctx.available_rect().width();
    egui::SidePanel::left("left_panel")
        .default_width(available_width / 3.)
        .resizable(true)
        .show(ctx, |ui| {
            ui.child_ui(ui.max_rect(), Layout::default());
            ui.add(
                &mut GraphView::<_, _, _, _, DefaultNodeShape, DefaultEdgeShape>::new(
                    &mut graph_provider.g,
                )
                .with_navigations(&SettingsNavigation::default().with_fit_to_screen_enabled(false)),
            )
        });
    egui::SidePanel::right("right_panel")
        .default_width(available_width / 3.)
        .resizable(true)
        .show(ctx, |ui| {
            ui.add(
                &mut GraphView::<_, _, _, _, DefaultNodeShape, DefaultEdgeShape>::new(
                    &mut graph_provider.g,
                )
                .with_navigations(&SettingsNavigation::default().with_fit_to_screen_enabled(false)),
            )
        });
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.add(
            &mut GraphView::<_, _, _, _, DefaultNodeShape, DefaultEdgeShape>::new(
                &mut graph_provider.g,
            )
            .with_navigations(&SettingsNavigation::default().with_fit_to_screen_enabled(false)),
        )
    });
}
