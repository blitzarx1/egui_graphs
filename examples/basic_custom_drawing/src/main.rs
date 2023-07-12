use eframe::{run_native, App, CreationContext};
use egui::{epaint::RectShape, Color32, Context, Pos2, Rect, Rounding, Stroke, Vec2};
use egui_graphs::{to_input_graph, Graph, GraphView, ShapesNodes};
use petgraph::{stable_graph::StableGraph, Directed};

pub struct BasicCustomDrawingApp {
    g: Graph<(), (), Directed>,
}

impl BasicCustomDrawingApp {
    fn new(_: &CreationContext<'_>) -> Self {
        let g = generate_graph();
        Self { g }
    }
}

impl App for BasicCustomDrawingApp {
    fn update(&mut self, ctx: &Context, _: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add(
                &mut GraphView::new(&mut self.g).with_custom_node_drawing(Some(
                    |res, loc, _node, comp_node| {
                        let color = Color32::from_rgb(128, 128, 255);
                        let size = Vec2::new(-comp_node.radius, -comp_node.radius);
                        let shape = RectShape {
                            rect: Rect::from_two_pos(loc + size / 2., loc - size / 2.),
                            fill: color,
                            stroke: Stroke::new(1., color),
                            rounding: Rounding::none(),
                        };
                        res.0 .0.push(shape.into());
                    },
                )),
            );
        });
    }
}

fn generate_graph() -> Graph<(), (), Directed> {
    let mut g: StableGraph<(), ()> = StableGraph::new();

    let a = g.add_node(());
    let b = g.add_node(());
    let c = g.add_node(());

    g.add_edge(a, b, ());
    g.add_edge(b, c, ());
    g.add_edge(c, a, ());

    to_input_graph(&g)
}

fn main() {
    let native_options = eframe::NativeOptions::default();
    run_native(
        "egui_graphs_basic_demo",
        native_options,
        Box::new(|cc| Box::new(BasicCustomDrawingApp::new(cc))),
    )
    .unwrap();
}
