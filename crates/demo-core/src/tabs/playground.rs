use crate::{DemoApp, DemoGraph};
use egui::{CollapsingHeader, Modal, ScrollArea, Ui};

impl DemoApp {
    pub fn ui_playground_tab(&mut self, ui: &mut Ui) {
        // Current content from the right side panel
        ScrollArea::vertical().show(ui, |ui| {
            ui.horizontal(|ui| {
                if ui.button("Export").on_hover_text("Export graph + optional layout to JSON").clicked() {
                    self.show_export_modal = true;
                }
                if ui
                    .button("Reset Defaults")
                .on_hover_text("Reset ALL settings, graph, layout & view state (Space)")
                    .clicked()
                {
                    self.reset_all(ui);
                }
            });

            // Export modal
            let export_modal = Modal::new(egui::Id::new("export_modal"));
            if self.show_export_modal {
                export_modal.show(ui.ctx(), |ui| {
                    ui.heading("Export Settings");
                    ui.separator();
                    ui.horizontal(|ui| {
                        ui.checkbox(&mut self.export_include_layout, "Include Layout");
                        ui.small_button("ℹ").on_hover_text(
                            "Exports the current layout choice and its parameters:\n\n- Fruchterman-Reingold: simulation params and CenterGravity\n- Hierarchical: row/col distances, centering, orientation",
                        );
                    });
                    ui.horizontal(|ui| {
                        ui.checkbox(&mut self.export_include_graph, "Include Graph");
                        ui.small_button("ℹ").on_hover_text(
                            "Exports the graph topology: node ids and edges.\nDirected flag controls edge orientation in the file.",
                        );
                    });
                    ui.horizontal(|ui| {
                        ui.checkbox(&mut self.export_include_positions, "Node Positions");
                        ui.small_button("ℹ").on_hover_text(
                            "When enabled, also exports each node's current (x,y) position as floats.",
                        );
                    });
                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        if ui.button("Cancel").clicked() {
                            self.show_export_modal = false;
                        }
                        if ui.button("Export").clicked() {
                            // Gather graph topology
                            let (nodes, edges, positions, directed) = match &self.g {
                                DemoGraph::Directed(g) => {
                                    let nodes: Vec<i64> = g
                                        .g()
                                        .node_indices()
                                        .map(|i| i.index() as i64)
                                        .collect();
                                    let edges: Vec<(i64, i64)> = g
                                        .g()
                                        .edge_indices()
                                        .filter_map(|e| g.g().edge_endpoints(e))
                                        .map(|(a, b)| (a.index() as i64, b.index() as i64))
                                        .collect();
                                    let positions = if self.export_include_positions {
                                        let mut v = Vec::with_capacity(nodes.len());
                                        for idx in g.g().node_indices() {
                                            let p = g.g().node_weight(idx).unwrap().location();
                                            v.push((idx.index() as i64, p.x, p.y));
                                        }
                                        Some(v)
                                    } else { None };
                                    (nodes, edges, positions, true)
                                }
                                DemoGraph::Undirected(g) => {
                                    let nodes: Vec<i64> = g
                                        .g()
                                        .node_indices()
                                        .map(|i| i.index() as i64)
                                        .collect();
                                    let edges: Vec<(i64, i64)> = g
                                        .g()
                                        .edge_indices()
                                        .filter_map(|e| g.g().edge_endpoints(e))
                                        .map(|(a, b)| {
                                            let (u, v) = if a.index() <= b.index() { (a, b) } else { (b, a) };
                                            (u.index() as i64, v.index() as i64)
                                        })
                                        .collect();
                                    let positions = if self.export_include_positions {
                                        let mut v = Vec::with_capacity(nodes.len());
                                        for idx in g.g().node_indices() {
                                            let p = g.g().node_weight(idx).unwrap().location();
                                            v.push((idx.index() as i64, p.x, p.y));
                                        }
                                        Some(v)
                                    } else { None };
                                    (nodes, edges, positions, false)
                                }
                            };
                            let spec = crate::spec::build_export_spec(
                                ui,
                                self.export_include_layout,
                                self.export_include_graph,
                                directed,
                                self.selected_layout,
                                nodes,
                                edges,
                                positions,
                            );
                            match serde_json::to_string_pretty(&spec) {
                                Ok(json) => {
                                    #[cfg(not(target_arch = "wasm32"))]
                                    {
                                        let file = rfd::FileDialog::new()
                                            .add_filter("JSON", &["json"])
                                            .set_file_name("graph_export.json")
                                            .save_file();
                                        if let Some(path) = file {
                                            if let Err(e) = std::fs::write(&path, json.as_bytes()) {
                                                self.status.push_error(format!("Export error: {}", e));
                                            } else {
                                                self.status.push_success(String::from("Exported to file"));
                                            }
                                        }
                                    }
                                    #[cfg(target_arch = "wasm32")]
                                    {
                                            ui.ctx().copy_text(json);
                                        self.status.push_success(String::from("Export copied to clipboard"));
                                    }
                                }
                                Err(e) => self.status.push_error(format!("Export serialize error: {}", e)),
                            }
                            self.show_export_modal = false;
                        }
                    });
                });
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
