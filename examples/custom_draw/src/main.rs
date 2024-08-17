use eframe::{run_native, App, CreationContext};
use egui::Context;
use egui_graphs::{DefaultEdgeShape, Graph, GraphView, SettingsInteraction, SettingsNavigation};
use node::NodeShapeAnimated;
use petgraph::{
    stable_graph::{DefaultIx, StableGraph},
    Directed,
};

mod node;

pub struct CustomDrawApp {
    g: Graph<node::NodeData, (), Directed, DefaultIx, NodeShapeAnimated>,
}

impl CustomDrawApp {
    fn new(_: &CreationContext<'_>) -> Self {
        let g = generate_graph();
        Self { g: Graph::from(&g) }
    }
}

impl App for CustomDrawApp {
    fn update(&mut self, ctx: &Context, _: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add(
                &mut GraphView::<_, _, _, _, NodeShapeAnimated, DefaultEdgeShape>::new(&mut self.g)
                    .with_navigations(
                        &SettingsNavigation::default()
                            .with_fit_to_screen_enabled(false)
                            .with_zoom_and_pan_enabled(true),
                    )
                    .with_interactions(
                        &SettingsInteraction::default()
                            .with_dragging_enabled(true)
                            .with_node_selection_enabled(true)
                            .with_edge_selection_enabled(true),
                    ),
            );
        });
    }
}

fn generate_graph() -> StableGraph<node::NodeData, ()> {
    let mut g = StableGraph::new();

    let a = g.add_node(node::NodeData { clockwise: true });
    let b = g.add_node(node::NodeData { clockwise: false });
    let c = g.add_node(node::NodeData { clockwise: false });

    g.add_edge(a, b, ());
    g.add_edge(b, c, ());
    g.add_edge(c, a, ());

    g
}

fn main() {
    let native_options = eframe::NativeOptions::default();
    run_native(
        "egui_graphs_custom_draw_demo",
        native_options,
        Box::new(|cc| Ok(Box::new(CustomDrawApp::new(cc)))),
    )
    .unwrap();
}
