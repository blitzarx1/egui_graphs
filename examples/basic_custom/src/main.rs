use eframe::{run_native, App, CreationContext, NativeOptions};
use egui::{Context, Pos2};
use egui_graphs::{DefaultGraphView, Graph, SettingsStyle};
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
            let idx = g.add_node_with_label_and_location((), position.to_string(), position);

            idxs.push(idx);
        }

        g.add_edge(idxs[0], idxs[1], ());
        g.add_edge(idxs[1], idxs[2], ());
        g.add_edge(idxs[2], idxs[0], ());

        Self { g }
    }
}

impl App for BasicCustomApp {
    fn update(&mut self, ctx: &Context, _: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add(
                &mut DefaultGraphView::new(&mut self.g)
                    .with_styles(&SettingsStyle::default().with_labels_always(true)),
            );
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
