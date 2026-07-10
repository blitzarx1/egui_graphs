use eframe::{run_native, App, CreationContext, NativeOptions};
use egui_graphs::{
    generate_simple_digraph, DefaultGraphView, Graph, GraphChange, SettingsInteraction,
    SettingsNavigation,
};

pub struct GraphViewResponseApp {
    graph: Graph,
    last_response: ResponseSnapshot,
    latest_changes: Vec<GraphChange>,
}

#[derive(Default)]
struct ResponseSnapshot {
    hovered: bool,
    contains_pointer: bool,
    clicked: bool,
    double_clicked: bool,
    dragged: bool,
    drag_started: bool,
    drag_stopped: bool,
}

impl ResponseSnapshot {
    fn update(&mut self, response: &egui::Response) {
        self.hovered = response.hovered();
        self.contains_pointer = response.contains_pointer();
        self.clicked = response.clicked();
        self.double_clicked = response.double_clicked();
        self.dragged = response.dragged();
        self.drag_started = response.drag_started();
        self.drag_stopped = response.drag_stopped();
    }
}

impl GraphViewResponseApp {
    fn new(_: &CreationContext<'_>) -> Self {
        Self {
            graph: Graph::from(&generate_simple_digraph()),
            last_response: ResponseSnapshot::default(),
            latest_changes: Vec::new(),
        }
    }
}

impl App for GraphViewResponseApp {
    fn ui(&mut self, ui: &mut egui::Ui, _: &mut eframe::Frame) {
        egui::Panel::right("graph_view_response")
            .default_size(360.0)
            .show(ui, |ui| {
                ui.heading("GraphViewResponse");
                ui.label(
                    "Interact with the graph to inspect the result of the previous show call.",
                );
                ui.separator();

                ui.strong("egui::Response");
                ui.monospace(format!("hovered: {}", self.last_response.hovered));
                ui.monospace(format!(
                    "contains_pointer: {}",
                    self.last_response.contains_pointer
                ));
                ui.monospace(format!("clicked: {}", self.last_response.clicked));
                ui.monospace(format!(
                    "double_clicked: {}",
                    self.last_response.double_clicked
                ));
                ui.monospace(format!("dragged: {}", self.last_response.dragged));
                ui.monospace(format!("drag_started: {}", self.last_response.drag_started));
                ui.monospace(format!("drag_stopped: {}", self.last_response.drag_stopped));

                ui.separator();
                ui.strong("Latest non-empty changes batch");
                if self.latest_changes.is_empty() {
                    ui.label("No graph changes yet.");
                } else {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        for (sequence, change) in self.latest_changes.iter().enumerate() {
                            ui.monospace(format!("{sequence}: {change:?}"));
                        }
                    });
                }
            });

        egui::CentralPanel::default().show(ui, |ui| {
            ui.label(
                "Click, double-click, select, or drag graph entities. Drag empty space to pan; Ctrl/Cmd + scroll to zoom.",
            );

            let result = DefaultGraphView::new()
                .with_interactions(
                    &SettingsInteraction::default()
                        .with_dragging_enabled(true)
                        .with_node_selection_enabled(true)
                        .with_edge_selection_enabled(true),
                )
                .with_navigations(
                    &SettingsNavigation::default()
                        .with_fit_to_screen_enabled(false)
                        .with_zoom_and_pan_enabled(true),
                )
                .show(ui, &mut self.graph);

            self.last_response.update(&result.response);
            if !result.changes.is_empty() {
                // GraphView changes are transient. Persist the latest interesting batch in app state.
                self.latest_changes = result.changes;
            }
        });
    }
}

fn main() {
    run_native(
        "graph_view_response",
        NativeOptions::default(),
        Box::new(|cc| Ok(Box::new(GraphViewResponseApp::new(cc)))),
    )
    .unwrap();
}
