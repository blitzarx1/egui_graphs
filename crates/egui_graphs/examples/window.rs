use eframe::{run_native, App, CreationContext};
use egui::{Context, Window};
use egui_graphs::{generate_simple_digraph, DefaultGraphView, Graph};

pub struct WindowApp {
    g: Graph,
}

impl WindowApp {
    fn new(_: &CreationContext<'_>) -> Self {
        let g = generate_simple_digraph();
        Self { g: Graph::from(&g) }
    }
}

impl App for WindowApp {
    fn update(&mut self, ctx: &Context, _: &mut eframe::Frame) {
        Window::new("windowed graph").show(ctx, |ui| {
            ui.add(&mut DefaultGraphView::new(&mut self.g));
        });
    }
}

fn main() {
    let native_options = eframe::NativeOptions::default();
    run_native(
        "window",
        native_options,
        Box::new(|cc| Ok(Box::new(WindowApp::new(cc)))),
    )
    .unwrap();
}
