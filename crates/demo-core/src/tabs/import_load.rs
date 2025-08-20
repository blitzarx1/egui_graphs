use crate::{DemoApp, DemoGraph};
use egui::Ui;

const SCHEMA_NODES: &str = r#"[
    [0,1], [1,2], [2,0]
]"#;
const SCHEMA_OBJECT: &str = r#"{
    "nodes": [0,1,2],
    "edges": [[0,1],[1,2],[2,0]]
}"#;
const SCHEMA_OBJECT_WITH_POSITIONS: &str = r#"{
    "nodes": [0,1,2],
    "edges": [[0,1],[1,2],[2,0]],
    "positions": [[0, 10.0, -5.0], [1, 0.0, 0.0], [2, 5.0, 12.5]]
}"#;
const SCHEMA_FULL: &str = r#"{
    "version": 1,
    "graph": { 
        "nodes": [0,1,2], 
        "edges": [[0,1],[1,2],[2,0]], 
        "directed": true 
    },
    "layout": {
        "type": "FruchtermanReingold",
        "running": true,
        "dt": 0.05,
        "k_scale": 1.2,
        "extras": [ { "type": "CenterGravity", "enabled": true, "c": 0.3 } ]
    }
}"#;

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

        // 1) User Uploads (drag & drop + Upload button)
        egui::CollapsingHeader::new("User Uploads")
            .default_open(true)
            .show(ui, |ui| {
                if self.user_uploads.is_empty() {
                    ui.weak("No uploads yet. Drag & drop a JSON file into the graph area or use the Upload button");
                } else {
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
                }

                // Open button placed below the note/grid and above the tip
                ui.add_space(6.0);
                if ui.button("Open").clicked() {
                    #[cfg(target_arch = "wasm32")]
                    {
                        // Use a native browser file input to avoid rfd's intermediate dialog on web.
                        let buf = self.web_upload_buf.clone();
                        pick_json_file_web(buf);
                    }
                    #[cfg(not(target_arch = "wasm32"))]
                    {
                        if let Some(path) = rfd::FileDialog::new()
                            .add_filter("JSON", &["json"])
                            .pick_file()
                        {
                            let name = path
                                .file_name()
                                .and_then(|o| o.to_str())
                                .unwrap_or("upload.json")
                                .to_owned();
                            match std::fs::read(&path) {
                                Ok(bytes) => match String::from_utf8(bytes) {
                                    Ok(text) => {
                                        // Reuse import flow and store in uploads (cap 20)
                                        self.load_graph_from_str(&name, &text);
                                        self.user_uploads.push(UserUpload { name, data: text });
                                        if self.user_uploads.len() > 20 {
                                            let overflow = self.user_uploads.len() - 20;
                                            self.user_uploads.drain(0..overflow);
                                        }
                                    }
                                    Err(e) => self
                                        .status
                                        .push_error(format!("Upload error (utf8): {}", e)),
                                },
                                Err(e) => self
                                    .status
                                    .push_error(format!("Upload error (read): {}", e)),
                            }
                        }
                    }
                }

                // On web, drain any completed async uploads and import them now
                #[cfg(target_arch = "wasm32")]
                {
                    // Take pending uploads out first to avoid borrowing self while calling methods
                    let pending: Vec<UserUpload> = {
                        let mut buf = self.web_upload_buf.borrow_mut();
                        core::mem::take(&mut *buf)
                    };
                    for up in pending.into_iter() {
                        self.load_graph_from_str(&up.name, &up.data);
                        self.user_uploads.push(up);
                        if self.user_uploads.len() > 20 {
                            let overflow = self.user_uploads.len() - 20;
                            self.user_uploads.drain(0..overflow);
                        }
                    }
                }

                ui.add_space(8.0);
                ui.group(|ui| {
                    ui.colored_label(
                        egui::Color32::from_rgb(200, 180, 40),
                        "Tip: User uploads are only available in the current session and will be lost when the session ends.",
                    );
                });
            });

        ui.add_space(6.0);

        // 2) Load assets from bundled examples
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

        // 3) Schema help (compact)
        egui::CollapsingHeader::new("Schema Help (JSON)")
            .default_open(true)
            .show(ui, |ui| {
                ui.add_space(8.0);
                ui.monospace("Edges array (directed implied):");
                ui.code(SCHEMA_NODES);

                ui.add_space(8.0);
                ui.monospace("Object form:");
                ui.code(SCHEMA_OBJECT);

                ui.add_space(8.0);
                ui.monospace("Optional positions (id, x, y) can be included in the graph object:");
                ui.code(SCHEMA_OBJECT_WITH_POSITIONS);

                ui.add_space(8.0);
                ui.monospace("Full schema with layout and graph options:");
                ui.code(SCHEMA_FULL);
            });
        ui.add_space(6.0);
    }

    pub fn load_graph_from_str(&mut self, name: &str, data: &str) {
        match crate::import::import_graph_from_str(data) {
            Ok(mut res) => {
                let applied_positions = res.positions_applied;
                match &mut res.g {
                    crate::import::ImportedGraph::Directed(g) => {
                        if !applied_positions {
                            Self::distribute_nodes_circle_generic(g);
                        }
                        self.g = DemoGraph::Directed(g.clone());
                        if let Some(pl) = res.pending_layout.take() {
                            self.pending_layout = Some(pl);
                            self.selected_layout = match self.pending_layout {
                                Some(crate::spec::PendingLayout::FR(_)) => {
                                    crate::DemoLayout::FruchtermanReingold
                                }
                                Some(crate::spec::PendingLayout::Hier(_)) => {
                                    crate::DemoLayout::Hierarchical
                                }
                                None => self.selected_layout,
                            };
                        }
                    }
                    crate::import::ImportedGraph::Undirected(g) => {
                        if !applied_positions {
                            Self::distribute_nodes_circle_generic(g);
                        }
                        self.g = DemoGraph::Undirected(g.clone());
                        if let Some(pl) = res.pending_layout.take() {
                            self.pending_layout = Some(pl);
                            self.selected_layout = match self.pending_layout {
                                Some(crate::spec::PendingLayout::FR(_)) => {
                                    crate::DemoLayout::FruchtermanReingold
                                }
                                Some(crate::spec::PendingLayout::Hier(_)) => {
                                    crate::DemoLayout::Hierarchical
                                }
                                None => self.selected_layout,
                            };
                        }
                    }
                }
                self.sync_counts();
                let (kind, n, e) = match &self.g {
                    DemoGraph::Directed(g) => ("directed", g.node_count(), g.edge_count()),
                    DemoGraph::Undirected(g) => ("undirected", g.node_count(), g.edge_count()),
                };
                let suffix = if applied_positions {
                    " (positions applied)"
                } else {
                    ""
                };
                self.status.push_success(format!(
                    "Loaded {} graph: {} nodes, {} edges{}",
                    kind, n, e, suffix
                ));
            }
            Err(err) => {
                self.status
                    .push_error(format!("Import error ({}): {}", name, err));
            }
        }
    }
}

// Web-only file picker implemented with a hidden <input type="file"> to avoid extra dialogs.
#[cfg(target_arch = "wasm32")]
fn pick_json_file_web(buf: std::rc::Rc<std::cell::RefCell<Vec<UserUpload>>>) {
    use wasm_bindgen::closure::Closure;
    use wasm_bindgen::JsCast;
    use web_sys::{window, FileReader, HtmlInputElement, ProgressEvent};

    let document = match window().and_then(|w| w.document()) {
        Some(d) => d,
        None => return,
    };

    // Create hidden input element
    let input: HtmlInputElement = match document.create_element("input") {
        Ok(el) => match el.dyn_into::<HtmlInputElement>() {
            Ok(i) => i,
            Err(_) => return,
        },
        Err(_) => return,
    };
    input.set_type("file");
    input.set_accept(".json,application/json");
    input.set_hidden(true);

    // Keep a clone for the change handler
    let input_clone = input.clone();
    let body = match document.body() {
        Some(b) => b,
        None => return,
    };
    // Attach to DOM so the click works reliably across browsers
    if body.append_child(&input).is_err() {
        return;
    }

    // When user selects a file, read it as text and push to buffer
    let change_cb = Closure::wrap(Box::new(move || {
        if let Some(files) = input_clone.files() {
            if let Some(file) = files.get(0) {
                let name = file.name();
                let reader = FileReader::new().unwrap();
                let reader_c = reader.clone();
                let buf_c = buf.clone();
                let name_rc = std::rc::Rc::new(name);
                let onloadend = Closure::wrap(Box::new(move |_e: ProgressEvent| {
                    if let Ok(result) = reader_c.result() {
                        if let Some(text) = result.as_string() {
                            let name_str: String = (*name_rc).clone();
                            buf_c.borrow_mut().push(UserUpload {
                                name: name_str,
                                data: text,
                            });
                        }
                    }
                }) as Box<dyn FnMut(_)>);
                reader.set_onloadend(Some(onloadend.as_ref().unchecked_ref()));
                // Start reading; ignore errors silently
                let _ = reader.read_as_text(&file);
                onloadend.forget();
            }
        }
        // Remove the input from DOM after use
        if let Some(parent) = input_clone.parent_node() {
            let _ = parent.remove_child(&input_clone);
        }
    }) as Box<dyn FnMut()>);

    let _ = input.add_event_listener_with_callback("change", change_cb.as_ref().unchecked_ref());
    change_cb.forget();

    // Trigger the picker
    let _ = input.click();
}
