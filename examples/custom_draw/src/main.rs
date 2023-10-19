use eframe::{run_native, App, CreationContext};
use egui::{
    epaint::TextShape, Context, FontFamily, FontId, Pos2, Rect, Rounding, Shape, Stroke, Vec2,
};
use egui_graphs::{Graph, GraphView};
use petgraph::{stable_graph::StableGraph, Directed};

pub struct BasicApp {
    g: Graph<(), (), Directed>,
}

impl BasicApp {
    fn new(_: &CreationContext<'_>) -> Self {
        let g = generate_graph();
        Self { g: Graph::from(&g) }
    }
}

impl App for BasicApp {
    fn update(&mut self, ctx: &Context, _: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add(&mut GraphView::new(&mut self.g).with_custom_node_draw(
                |ctx, n, meta, style, l| {
                    // lets draw a rect with label in the center for every node

                    // find node center location on the screen coordinates
                    let node_center_loc = n.screen_location(meta).to_pos2();

                    // find node radius accounting for current zoom level; we will use it as a reference for the rect and label sizes
                    let rad = n.screen_radius(meta, style);

                    // first create rect shape
                    let size = Vec2::new(rad * 1.5, rad * 1.5);
                    let rect = Rect::from_center_size(node_center_loc, size);
                    let shape_rect = Shape::rect_stroke(
                        rect,
                        Rounding::default(),
                        Stroke::new(1., n.color(ctx)),
                    );

                    // then create shape for the label placing it in the center of the rect
                    let color = ctx.style().visuals.text_color();
                    let galley = ctx.fonts(|f| {
                        f.layout_no_wrap(n.label(), FontId::new(rad, FontFamily::Monospace), color)
                    });
                    // we need to offset a bit to place the label in the center of the rect
                    let label_loc =
                        Pos2::new(node_center_loc.x - rad / 2., node_center_loc.y - rad / 2.);
                    let shape_label = TextShape::new(label_loc, galley);

                    // add shapes to the drawing layers; the drawing process is happening in the widget lifecycle.
                    l.add(shape_rect);
                    l.add(shape_label);
                },
            ));
        });
    }
}

fn generate_graph() -> StableGraph<(), (), Directed> {
    let mut g: StableGraph<(), ()> = StableGraph::new();

    let a = g.add_node(());
    let b = g.add_node(());
    let c = g.add_node(());

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
        Box::new(|cc| Box::new(BasicApp::new(cc))),
    )
    .unwrap();
}
