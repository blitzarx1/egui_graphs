use eframe::{run_native, App, CreationContext};
use egui::{
    epaint::TextShape, Context, FontFamily, FontId, Rect, Rounding, Shape, Stroke, Vec2,
};
use egui_graphs::{default_edges_draw, Graph, GraphView, SettingsInteraction};
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
            ui.add(
                &mut GraphView::new(&mut self.g)
                    .with_interactions(
                        &SettingsInteraction::default()
                            .with_dragging_enabled(true)
                            .with_node_selection_enabled(true),
                    )
                    .with_custom_node_draw(|ctx, n, state, l| {
                        // lets draw a rect with label in the center for every node

                        // find node center location on the screen coordinates
                        let node_center_loc = n.screen_location(state.meta).to_pos2();

                        // find node radius accounting for current zoom level; we will use it as a reference for the rect and label sizes
                        let rad = n.screen_radius(state.meta, state.style);

                        // first create rect shape
                        let size = Vec2::new(rad * 1.5, rad * 1.5);
                        let rect = Rect::from_center_size(node_center_loc, size);
                        let shape_rect = Shape::rect_stroke(
                            rect,
                            Rounding::default(),
                            Stroke::new(1., n.color(ctx)),
                        );

                        // add rect to the layers
                        l.add(shape_rect);

                        // then create label
                        let color = ctx.style().visuals.text_color();
                        let galley = ctx.fonts(|f| {
                            f.layout_no_wrap(
                                n.label(),
                                FontId::new(rad, FontFamily::Monospace),
                                color,
                            )
                        });

                        // we need to offset label by half its size to place it in the center of the rect
                        let offset = Vec2::new(-galley.size().x / 2., -galley.size().y / 2.);

                        // create the shape and add it to the layers
                        let shape_label = TextShape::new(node_center_loc + offset, galley);
                        l.add(shape_label);
                    })
                    .with_custom_edge_draw(|ctx, bounds, edges, state, l| {
                        // draw edges with labels in the middle

                        // draw default edges
                        default_edges_draw(ctx, bounds, edges, state, l);

                        // get start and end nodes
                        let n_start = state.g.node(bounds.0).unwrap();
                        let n_end = state.g.node(bounds.1).unwrap();

                        // get start and end node locations
                        let loc_start = n_start.screen_location(state.meta);
                        let loc_end = n_end.screen_location(state.meta);

                        // compute edge center location
                        let center_loc = (loc_start + loc_end) / 2.;

                        // let label be the average of bound nodes sizes
                        let size = (n_start.screen_radius(state.meta, state.style)
                            + n_end.screen_radius(state.meta, state.style))
                            / 2.;

                        // create label
                        let color = ctx.style().visuals.text_color();
                        let galley = ctx.fonts(|f| {
                            f.layout_no_wrap(
                                format!("{}->{}", n_start.label(), n_end.label()),
                                FontId::new(size, FontFamily::Monospace),
                                color,
                            )
                        });

                        // we need to offset half the label size to place it in the center of the edge
                        let offset = Vec2::new(-galley.size().x / 2., -galley.size().y / 2.);

                        // create the shape and add it to the layers
                        let shape_label = TextShape::new((center_loc + offset).to_pos2(), galley);
                        l.add(shape_label);
                    }),
            );
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
