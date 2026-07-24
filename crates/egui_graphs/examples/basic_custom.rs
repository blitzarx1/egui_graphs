use eframe::{run_native, App, CreationContext, NativeOptions};
use egui::Pos2;
use egui_graphs::{DefaultGraphView, Graph, SettingsStyle};

pub struct BasicCustomApp {
    g: Graph,
}

impl BasicCustomApp {
    fn new(_: &CreationContext<'_>) -> Self {
        let mut g = Graph::new();

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
    fn ui(&mut self, ui: &mut egui::Ui, _: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ui, |ui| {
            DefaultGraphView::new()
                .with_styles(&SettingsStyle::default().with_labels_always(true))
                .show(ui, &mut self.g);
        });
    }
}

fn main() {
    let native_options = NativeOptions::default();
    run_native(
        "basic_custom",
        native_options,
        Box::new(|cc| Ok(Box::new(BasicCustomApp::new(cc)))),
    )
    .unwrap();
}
