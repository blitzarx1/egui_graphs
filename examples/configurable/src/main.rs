fn main() {
    use configurable::ConfigurableApp;
    use eframe::run_native;

    let native_options = eframe::NativeOptions::default();
    run_native(
        "egui_graphs_configurable_demo",
        native_options,
        Box::new(|cc| Box::new(ConfigurableApp::new(cc))),
    )
    .unwrap();
}
