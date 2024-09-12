use eframe::{run_native, App, CreationContext};
use egui::Context;
use egui_graphs::{
    default_edge_transform, default_node_transform, to_graph_custom, DefaultEdgeShape, Graph,
    GraphView, SettingsInteraction, SettingsNavigation,
};
use node::NodeShapeAnimated;
use petgraph::{
    stable_graph::{DefaultIx, StableGraph},
    Directed,
};

mod node;

const GLYPH_CLOCKWISE: &str = "↻";
const GLYPH_ANTICLOCKWISE: &str = "↺";

pub struct AnimatedNodesApp {
    g: Graph<node::NodeData, (), Directed, DefaultIx, NodeShapeAnimated>,
}

impl AnimatedNodesApp {
    fn new(_: &CreationContext<'_>) -> Self {
        let g = generate_graph();
        Self {
            g: to_graph_custom(
                &g,
                |idx, n| {
                    if n.clockwise {
                        default_node_transform(idx, n).with_label(GLYPH_CLOCKWISE.to_string())
                    } else {
                        default_node_transform(idx, n).with_label(GLYPH_ANTICLOCKWISE.to_string())
                    }
                },
                default_edge_transform,
            ),
        }
    }
}

impl App for AnimatedNodesApp {
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
        "egui_graphs_animated_nodes_demo",
        native_options,
        Box::new(|cc| Ok(Box::new(AnimatedNodesApp::new(cc)))),
    )
    .unwrap();
}
