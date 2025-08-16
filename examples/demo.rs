fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "egui_graphs demo",
        native_options,
        Box::new(|cc| Ok::<Box<dyn eframe::App>, _>(Box::new(demo_core::DemoApp::new(cc)))),
    )
}
