use eframe::{run_native, App, CreationContext, Frame};
use egui::{CentralPanel, Context, Layout, SidePanel};
use egui_graphs::{generate_simple_digraph, DefaultGraphView, Graph};

pub struct BasicApp {
    g: Graph,
}

impl BasicApp {
    fn new(_: &CreationContext<'_>) -> Self {
        let g = generate_simple_digraph();
        Self { g: Graph::from(&g) }
    }
}

impl App for BasicApp {
    fn update(&mut self, ctx: &Context, _: &mut Frame) {
        let available_width = ctx.available_rect().width();
        SidePanel::left("left_panel")
            .default_width(available_width / 3.)
            .resizable(true)
            .show(ctx, |ui| {
                ui.allocate_ui_with_layout(ui.max_rect().size(), Layout::default(), |ui| {
                    ui.add(&mut DefaultGraphView::new(&mut self.g));
                });
            });
        SidePanel::right("right_panel")
            .default_width(available_width / 3.)
            .resizable(true)
            .show(ctx, |ui| ui.add(&mut DefaultGraphView::new(&mut self.g)));
        CentralPanel::default().show(ctx, |ui| ui.add(&mut DefaultGraphView::new(&mut self.g)));
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
