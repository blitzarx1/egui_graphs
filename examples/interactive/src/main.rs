use eframe::{run_native, App, CreationContext};
use egui::Context;
use egui_graphs::{
    DefaultGraphView, Graph, SettingsInteraction, SettingsNavigation, SettingsStyle,
};
use petgraph::stable_graph::StableGraph;

pub struct InteractiveApp {
    g: Graph,
}

impl InteractiveApp {
    fn new(_: &CreationContext<'_>) -> Self {
        let g: Graph = generate_graph();
        Self { g }
    }
}

impl App for InteractiveApp {
    fn update(&mut self, ctx: &Context, _: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let interaction_settings = &SettingsInteraction::new()
                .with_dragging_enabled(true)
                .with_node_clicking_enabled(true)
                .with_node_selection_enabled(true)
                .with_node_selection_multi_enabled(true)
                .with_edge_clicking_enabled(true)
                .with_edge_selection_enabled(true)
                .with_edge_selection_multi_enabled(true);
            let style_settings = &SettingsStyle::new().with_labels_always(true);
            let navigation_settings = &SettingsNavigation::new()
                .with_fit_to_screen_enabled(false)
                .with_zoom_and_pan_enabled(true);

            ui.add(
                &mut DefaultGraphView::new(&mut self.g)
                    .with_styles(style_settings)
                    .with_interactions(interaction_settings)
                    .with_navigations(navigation_settings),
            );
        });
    }
}

fn generate_graph() -> Graph {
    let mut g = StableGraph::new();

    let a = g.add_node(());
    let b = g.add_node(());
    let c = g.add_node(());

    g.add_edge(a, a, ());
    g.add_edge(a, b, ());
    g.add_edge(a, b, ());
    g.add_edge(c, b, ());
    g.add_edge(b, c, ());
    g.add_edge(c, a, ());

    Graph::from(&g)
}

fn main() {
    let native_options = eframe::NativeOptions::default();
    run_native(
        "egui_graphs_interactive_demo",
        native_options,
        Box::new(|cc| Ok(Box::new(InteractiveApp::new(cc)))),
    )
    .unwrap();
}
