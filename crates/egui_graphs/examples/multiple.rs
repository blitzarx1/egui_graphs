// FIXME: when small screen width, the graph slow to respond to drag
// TODO: check that multiple views with same id work as expected (set custom zoom and pan and check sync)
// FIXME: graph is not visible for id_1, works fine when ids and graphs are different

use eframe::{run_native, App, CreationContext, Frame};
use egui::{CentralPanel, Context, Layout, SidePanel};
use egui_graphs::{generate_simple_digraph, DefaultGraphView, Graph};

pub struct BasicApp {
    g1: Graph,
    g2: Graph,
}

impl BasicApp {
    fn new(_: &CreationContext<'_>) -> Self {
        let g1 = generate_simple_digraph();
        let g2 = generate_simple_digraph();
        Self {
            g1: Graph::from(&g1),
            g2: Graph::from(&g2),
        }
    }
}

impl App for BasicApp {
    fn update(&mut self, ctx: &Context, _: &mut Frame) {
        let id1 = Some("id_1".to_string());
        let id2 = Some("id_2".to_string());

        let available_width = ctx.available_rect().width();
        SidePanel::left("left_panel")
            .default_width(available_width / 3.)
            .resizable(true)
            .show(ctx, |ui| {
                ui.allocate_ui_with_layout(ui.max_rect().size(), Layout::default(), |ui| {
                    ui.add(&mut DefaultGraphView::new(&mut self.g1).with_id(id1.clone()));
                });
            });
        SidePanel::right("right_panel")
            .default_width(available_width / 3.)
            .resizable(true)
            .show(ctx, |ui| {
                ui.add(&mut DefaultGraphView::new(&mut self.g1).with_id(id1))
            });
        CentralPanel::default().show(ctx, |ui| {
            ui.add(&mut DefaultGraphView::new(&mut self.g2).with_id(id2))
        });
    }
}

fn main() {
    let native_options = eframe::NativeOptions::default();
    run_native(
        "multiple",
        native_options,
        Box::new(|cc| Ok(Box::new(BasicApp::new(cc)))),
    )
    .unwrap();
}
