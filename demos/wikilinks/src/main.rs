use eframe::{run_native, App, CreationContext, Frame, NativeOptions};
use egui::{CentralPanel, Context};

const APP_NAME: &str = "Wiki Links";

mod app;
mod node;
mod state;
mod url;
mod url_retriever;

pub struct WikiLinksApp {
    app: app::App,
}

impl WikiLinksApp {
    fn new(_: &CreationContext<'_>) -> Self {
        Self {
            app: app::App::new(),
        }
    }
}

impl App for WikiLinksApp {
    fn update(&mut self, ctx: &Context, _: &mut Frame) {
        CentralPanel::default().show(ctx, |ui| self.app.update(ctx, ui));
    }
}

#[tokio::main]
async fn main() {
    env_logger::init();

    let native_options = NativeOptions::default();
    run_native(
        APP_NAME,
        native_options,
        Box::new(|cc| Box::new(WikiLinksApp::new(cc))),
    )
    .unwrap();
}
