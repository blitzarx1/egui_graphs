use crate::DemoApp;
use egui::{CollapsingHeader, ScrollArea, Ui};

impl DemoApp {
    pub fn ui_playground_tab(&mut self, ui: &mut Ui) {
        // Current content from the right side panel
        ScrollArea::vertical().show(ui, |ui| {
            if ui
                .button("Reset Defaults")
                .on_hover_text("Reset ALL settings, graph, layout & view state (Space)")
                .clicked()
            {
                self.reset_all(ui);
            }
            CollapsingHeader::new("Graph")
                .default_open(true)
                .show(ui, |ui| self.ui_graph_section(ui));
            self.ui_navigation(ui);
            self.ui_layout_section(ui);
            self.ui_layout_force_directed(ui);
            self.ui_interaction(ui);
            self.ui_selected(ui);
            self.ui_style(ui);
            self.ui_debug(ui);
            self.ui_events(ui);
        });
    }
}
