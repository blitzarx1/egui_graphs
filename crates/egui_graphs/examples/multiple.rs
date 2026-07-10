use eframe::{run_native, App, CreationContext, Frame};
use egui::{CentralPanel, Panel, RichText};
use egui_graphs::{
    generate_simple_digraph, DefaultGraphView, Graph, SettingsInteraction, SettingsNavigation,
};

const WITH_ID_DOCS_URL: &str = concat!(
    "https://docs.rs/egui_graphs/",
    env!("CARGO_PKG_VERSION"),
    "/egui_graphs/struct.GraphView.html#method.with_id"
);

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
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut Frame) {
        let id1 = Some("id_1".to_string());
        let id2 = Some("id_2".to_string());

        Panel::top("info_panel").show(ui, render_info);

        let available_width = ui.ctx().content_rect().width();

        let settings_nav = SettingsNavigation::default()
            .with_fit_to_screen_enabled(false)
            .with_zoom_and_pan_enabled(true);
        let settings_int = SettingsInteraction::default()
            .with_node_selection_enabled(true)
            .with_edge_selection_enabled(true);
        Panel::left("left_panel")
            .default_size(available_width / 3.)
            .resizable(true)
            .show(ui, |ui| {
                let result = DefaultGraphView::new()
                    .with_navigations(&settings_nav)
                    .with_interactions(&settings_int)
                    .with_id(id1.clone())
                    .show(ui, &mut self.g1);
                render_graph_id(ui, &result.response, &id1);
            });
        Panel::right("right_panel")
            .default_size(available_width / 3.)
            .resizable(true)
            .show(ui, |ui| {
                let result = DefaultGraphView::new()
                    .with_navigations(&settings_nav)
                    .with_interactions(&settings_int)
                    .with_id(id1.clone())
                    .show(ui, &mut self.g1);
                render_graph_id(ui, &result.response, &id1);
            });
        CentralPanel::default().show(ui, |ui| {
            let result = DefaultGraphView::new()
                .with_navigations(&settings_nav)
                .with_interactions(&settings_int)
                .with_id(id2.clone())
                .show(ui, &mut self.g2);
            render_graph_id(ui, &result.response, &id2);
        });
    }
}

fn render_graph_id(ui: &egui::Ui, resp: &egui::Response, id: &Option<String>) {
    let id = id.as_deref().unwrap_or("default");
    let text = format!("Id {id}");
    let painter = ui.painter();
    let font_id = egui::FontId::monospace(12.0);
    let text_color = ui.visuals().strong_text_color();
    let padding = egui::vec2(6.0, 3.0);
    let pos = resp.rect.left_top() + egui::vec2(8.0, 8.0);
    let galley = painter.layout_no_wrap(text, font_id, text_color);
    let text_rect = egui::Rect::from_min_size(pos + padding, galley.size());

    painter.rect_filled(
        text_rect.expand2(padding),
        2.0,
        ui.visuals().extreme_bg_color,
    );
    painter.galley(text_rect.min, galley, text_color);
}

fn render_info(ui: &mut egui::Ui) {
    ui.group(|ui| {
        ui.vertical(|ui| {
            ui.horizontal_wrapped(|ui| {
                ui.label(RichText::new("Multiple GraphView instances").strong());
                ui.label("can either share interaction state or keep navigation local by");
                ui.label(RichText::new("Id").code());
                ui.label(".");
            });

            ui.horizontal_wrapped(|ui| {
                ui.label("*");
                ui.label(RichText::new("GraphView").code());
                ui.label("instances with the same");
                ui.label(RichText::new("Id").code());
                ui.label("share interaction state, including hover, drag, selection,");
                ui.label(RichText::new("zoom").code());
                ui.label("and");
                ui.label(RichText::new("pan").code());
                ui.label(".");
            });

            ui.horizontal_wrapped(|ui| {
                ui.label("*");
                ui.label(RichText::new("zoom").code());
                ui.label("and");
                ui.label(RichText::new("pan").code());
                ui.label("stay local when");
                ui.label(RichText::new("GraphView").code());
                ui.label("instances use different");
                ui.label(RichText::new("Id").code());
                ui.label("values, so each graph keeps independent navigation.");
            });

            ui.horizontal_wrapped(|ui| {
                ui.label("*");
                ui.label("A custom");
                ui.label(RichText::new("Id").code());
                ui.label("can be assigned with");
                ui.hyperlink_to(RichText::new("with_id").code(), WITH_ID_DOCS_URL);
                ui.label(".");
            });
        });
    });
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
