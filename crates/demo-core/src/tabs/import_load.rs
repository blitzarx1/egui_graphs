use crate::{DemoApp, DemoGraph};
use egui::Ui;

#[derive(Debug, Clone)]
pub struct UserUpload {
    pub name: String,
    pub data: String,
}

impl DemoApp {
    pub fn ui_import_tab(&mut self, ui: &mut Ui) {
        // Include the generated assets manifest (build.rs)
        #[allow(non_upper_case_globals)]
        mod assets_manifest {
            include!(concat!(env!("OUT_DIR"), "/assets_manifest.rs"));
        }

        // 1) User Uploads (from drag & drop)
        egui::CollapsingHeader::new("User Uploads")
            .default_open(true)
            .show(ui, |ui| {
                if self.user_uploads.is_empty() {
                    ui.weak("No uploads yet. Drag & drop a JSON file into the graph area.");
                    return;
                }
                egui::Grid::new("uploads_grid")
                    .striped(true)
                    .show(ui, |ui| {
                        let mut action: Option<(usize, bool)> = None; // (index, load=true/delete=false)
                        for (i, up) in self.user_uploads.iter().enumerate() {
                            ui.label(&up.name);
                            if ui.button("Load").clicked() {
                                action = Some((i, true));
                            }
                            if ui.button("Delete").clicked() {
                                action = Some((i, false));
                            }
                            ui.end_row();
                        }
                        if let Some((i, do_load)) = action {
                            if do_load {
                                let up = self.user_uploads[i].clone();
                                self.load_graph_from_str(&up.name, &up.data);
                            } else {
                                self.user_uploads.remove(i);
                            }
                        }
                    });
            });

        // 2) Schema help (compact)
        egui::CollapsingHeader::new("Schema Help (JSON)")
            .default_open(true)
            .show(ui, |ui| {
                ui.small("Supported minimal schemas:");
                ui.add_space(4.0);
                ui.monospace("Edges array (directed implied):");
                ui.code("[[0,1],[1,2],[2,0]]");
                ui.add_space(4.0);
                ui.monospace("Object form:");
                ui.code(r#"{ "nodes": [0,1,2], "edges": [[0,1],[1,2],[2,0]], "directed": false }"#);
                ui.add_space(4.0);
                ui.small("Notes: nodes are integers; edges are pairs [1,2]; set directed=false for undirected graphs");
            });

        ui.add_space(6.0);

        // 3) Load assets from bundled examples
        egui::CollapsingHeader::new("Example Graphs")
            .default_open(true)
            .show(ui, |ui| {
                egui::Grid::new("assets_grid").striped(true).show(ui, |ui| {
                    for (name, data) in assets_manifest::ASSETS.iter() {
                        ui.label(*name);
                        if ui.button("Load").clicked() {
                            self.load_graph_from_str(name, data);
                        }
                        ui.end_row();
                    }
                });
            });

        ui.add_space(6.0);
    }

    pub fn load_graph_from_str(&mut self, name: &str, data: &str) {
        match crate::import::import_graph_from_str(data) {
            Ok(mut res) => {
                match &mut res.g {
                    crate::import::ImportedGraph::Directed(g) => {
                        Self::distribute_nodes_circle_generic(g);
                        self.g = DemoGraph::Directed(g.clone());
                    }
                    crate::import::ImportedGraph::Undirected(g) => {
                        Self::distribute_nodes_circle_generic(g);
                        self.g = DemoGraph::Undirected(g.clone());
                    }
                }
                self.sync_counts();
                let (kind, n, e) = match &self.g {
                    DemoGraph::Directed(g) => ("directed", g.node_count(), g.edge_count()),
                    DemoGraph::Undirected(g) => ("undirected", g.node_count(), g.edge_count()),
                };
                self.status
                    .push_success(format!("Loaded {} graph: {} nodes, {} edges", kind, n, e));
            }
            Err(err) => {
                self.status
                    .push_error(format!("Import error ({}): {}", name, err));
            }
        }
    }
}
