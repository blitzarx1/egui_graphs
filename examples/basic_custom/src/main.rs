use eframe::{run_native, App, CreationContext, NativeOptions};
use egui::{Context, Pos2};
use egui_graphs::{add_edge, add_node_custom, Graph, GraphView};
use petgraph::stable_graph::StableGraph;

pub struct BasicCustomApp {
    g: Graph,
}

impl BasicCustomApp {
    fn new(_: &CreationContext<'_>) -> Self {
        let mut g = Graph::new(StableGraph::default());

        let positions = vec![Pos2::new(0., 0.), Pos2::new(50., 0.), Pos2::new(0., 50.)];
        let mut idxs = Vec::with_capacity(positions.len());
        for position in positions {
            idxs.push(add_node_custom(&mut g, &(), |g_node| {
                g_node.set_location(position);
            }));
        }

        add_edge(&mut g, idxs[0], idxs[1], &());
        add_edge(&mut g, idxs[1], idxs[2], &());
        add_edge(&mut g, idxs[2], idxs[0], &());

        Self { g }
    }
}

impl App for BasicCustomApp {
    fn update(&mut self, ctx: &Context, _: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add(&mut GraphView::new(&mut self.g));
        });
    }
}

fn main() {
    let native_options = NativeOptions::default();
    run_native(
        "egui_graphs_basic_custom_demo",
        native_options,
        Box::new(|cc| Ok(Box::new(BasicCustomApp::new(cc)))),
    )
    .unwrap();
}
