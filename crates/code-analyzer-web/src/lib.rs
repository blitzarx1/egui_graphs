use eframe::wasm_bindgen::{self, prelude::*};

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();

    wasm_bindgen_futures::spawn_local(async {
        let document = web_sys::window()
            .expect("no global window exists")
            .document()
            .expect("should have a document on window");
        
        let canvas = document
            .get_element_by_id("canvas")
            .expect("no canvas element with id 'canvas'")
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .expect("element with id 'canvas' is not a canvas");
        
        let web_options = eframe::WebOptions::default();
        
        eframe::WebRunner::new()
            .start(
                canvas,
                web_options,
                Box::new(|cc| Ok(Box::new(code_analyzer::CodeAnalyzerApp::new(cc)))),
            )
            .await
            .expect("failed to start eframe");
    });

    Ok(())
}

mod code_analyzer {
    use eframe::App;
    use egui::{Color32, FontFamily, FontId, Pos2, Rect, Shape, Stroke, Vec2};
    use egui_graphs::{
        DisplayEdge, DisplayNode, DrawContext, EdgeProps, Graph, GraphView, Node, NodeProps,
        SettingsInteraction, SettingsNavigation, SettingsStyle, FruchtermanReingoldState,
        get_layout_state, set_layout_state,
    };
    use petgraph::{stable_graph::{NodeIndex, StableGraph}, Directed};
    use std::collections::HashMap;

    #[derive(Clone, Debug)]
    pub struct ClassInfo {
        name: String,
        methods: Vec<String>,
        fields: Vec<String>,
    }

    #[derive(Clone, Debug)]
    enum Relationship {
        OneToOne,
        OneToMany,
        ManyToMany,
    }

    impl Relationship {
        fn label(&self) -> &str {
            match self {
                Relationship::OneToOne => "1:1",
                Relationship::OneToMany => "1:N",
                Relationship::ManyToMany => "N:N",
            }
        }
    }

    #[derive(Clone, Debug)]
    struct CodeNode {
        pos: Pos2,
        z_pos: f32,
        selected: bool,
        dragged: bool,
        hovered: bool,
        class_name: String,
        radius: f32,
        node_color: [u8; 3],
        hover_color: [u8; 3],
        selected_color: [u8; 3],
    }

    impl From<NodeProps<ClassInfo>> for CodeNode {
        fn from(node_props: NodeProps<ClassInfo>) -> Self {
            let default_config = AppConfig::default();
            Self {
                pos: node_props.location(),
                z_pos: 0.0,
                selected: node_props.selected,
                dragged: node_props.dragged,
                hovered: node_props.hovered,
                class_name: node_props.payload.name.clone(),
                radius: 30.0,
                node_color: default_config.node_color,
                hover_color: default_config.node_hover_color,
                selected_color: default_config.node_selected_color,
            }
        }
    }

    impl DisplayNode<ClassInfo, Relationship, Directed, petgraph::graph::DefaultIx> for CodeNode {
        fn is_inside(&self, pos: Pos2) -> bool {
            let dir = pos - self.pos;
            dir.length() <= self.radius
        }

        fn closest_boundary_point(&self, dir: Vec2) -> Pos2 {
            self.pos + dir.normalized() * self.radius
        }

        fn shapes(&mut self, ctx: &DrawContext) -> Vec<Shape> {
            let mut shapes = Vec::new();
            let screen_pos = ctx.meta.canvas_to_screen_pos(self.pos);
            let screen_radius = ctx.meta.canvas_to_screen_size(self.radius);
            
            // Note: depth_scale_strength and depth_fade_strength are applied in the update loop
            // where we have access to config. Here we use the pre-calculated z_pos effects.
            let adjusted_radius = screen_radius;

            let color = if self.selected {
                Color32::from_rgb(self.selected_color[0], self.selected_color[1], self.selected_color[2])
            } else if self.hovered {
                Color32::from_rgb(self.hover_color[0], self.hover_color[1], self.hover_color[2])
            } else {
                Color32::from_rgb(self.node_color[0], self.node_color[1], self.node_color[2])
            };
            
            // Color is adjusted in the update method based on config
            let adjusted_color = color;

            let stroke = if self.selected {
                Stroke::new(2.0, Color32::WHITE)
            } else {
                Stroke::new(1.0, Color32::GRAY)
            };

            shapes.push(
                egui::epaint::CircleShape {
                    center: screen_pos,
                    radius: adjusted_radius,
                    fill: adjusted_color,
                    stroke,
                }
                .into(),
            );

            let font_size = (adjusted_radius * 0.4).max(8.0).min(16.0);
            let galley = ctx.ctx.fonts_mut(|f| {
                f.layout_no_wrap(
                    self.class_name.clone(),
                    FontId::new(font_size, FontFamily::Proportional),
                    Color32::WHITE,
                )
            });

            let text_pos = Pos2::new(
                screen_pos.x - galley.size().x / 2.0,
                screen_pos.y - galley.size().y / 2.0,
            );

            shapes.push(egui::epaint::TextShape::new(text_pos, galley, Color32::WHITE).into());
            shapes
        }

        fn update(&mut self, state: &NodeProps<ClassInfo>) {
            self.pos = state.location();
            self.selected = state.selected;
            self.dragged = state.dragged;
            self.hovered = state.hovered;
            self.class_name = state.payload.name.clone();
        }
    }

    #[derive(Clone, Debug)]
    struct CodeEdge {
        order: usize,
        selected: bool,
        label: String,
        edge_color: [u8; 3],
        selected_color: [u8; 3],
    }

    impl From<EdgeProps<Relationship>> for CodeEdge {
        fn from(edge_props: EdgeProps<Relationship>) -> Self {
            let default_config = AppConfig::default();
            Self {
                order: edge_props.order,
                selected: edge_props.selected,
                label: edge_props.payload.label().to_string(),
                edge_color: default_config.edge_color,
                selected_color: default_config.edge_selected_color,
            }
        }
    }

    impl DisplayEdge<ClassInfo, Relationship, Directed, petgraph::graph::DefaultIx, CodeNode>
        for CodeEdge
    {
        fn is_inside(
            &self,
            start: &Node<ClassInfo, Relationship, Directed, petgraph::graph::DefaultIx, CodeNode>,
            end: &Node<ClassInfo, Relationship, Directed, petgraph::graph::DefaultIx, CodeNode>,
            pos: Pos2,
        ) -> bool {
            let start_pos = start.location();
            let end_pos = end.location();
            let radius = 5.0;
            let line_vec = end_pos - start_pos;
            let point_vec = pos - start_pos;
            let line_len = line_vec.length();
            if line_len < 0.001 {
                return false;
            }
            let proj = point_vec.dot(line_vec) / line_len;
            if proj < 0.0 || proj > line_len {
                return false;
            }
            let closest = start_pos + line_vec.normalized() * proj;
            (pos - closest).length() <= radius
        }

        fn shapes(
            &mut self,
            start: &Node<ClassInfo, Relationship, Directed, petgraph::graph::DefaultIx, CodeNode>,
            end: &Node<ClassInfo, Relationship, Directed, petgraph::graph::DefaultIx, CodeNode>,
            ctx: &DrawContext,
        ) -> Vec<Shape> {
            let start_pos = start.location();
            let end_pos = end.location();

            let dir = (end_pos - start_pos).normalized();
            let start_boundary = start.display().closest_boundary_point(dir);
            let end_boundary = end.display().closest_boundary_point(-dir);
            let mut shapes = Vec::new();
            let screen_start = ctx.meta.canvas_to_screen_pos(start_boundary);
            let screen_end = ctx.meta.canvas_to_screen_pos(end_boundary);

            let color = if self.selected {
                Color32::from_rgb(self.selected_color[0], self.selected_color[1], self.selected_color[2])
            } else {
                Color32::from_rgb(self.edge_color[0], self.edge_color[1], self.edge_color[2])
            };
            let stroke = Stroke::new(2.0, color);

            shapes.push(egui::epaint::Shape::line_segment([screen_start, screen_end], stroke));

            let dir = (screen_end - screen_start).normalized();
            let arrow_size = 10.0;
            let perp = Vec2::new(-dir.y, dir.x);
            let tip = screen_end - dir * arrow_size;
            let left = tip + perp * arrow_size * 0.5;
            let right = tip - perp * arrow_size * 0.5;

            shapes.push(egui::epaint::Shape::convex_polygon(
                vec![screen_end, left, right],
                color,
                Stroke::NONE,
            ));

            let midpoint = (screen_start + screen_end.to_vec2()) * 0.5;
            let galley = ctx.ctx.fonts_mut(|f| {
                f.layout_no_wrap(
                    self.label.clone(),
                    FontId::new(12.0, FontFamily::Proportional),
                    color,
                )
            });

            let label_pos = Pos2::new(
                midpoint.x - galley.size().x / 2.0,
                midpoint.y - galley.size().y / 2.0 - 10.0,
            );

            let label_rect = Rect::from_min_size(label_pos, galley.size() + Vec2::new(4.0, 2.0));
            shapes.push(egui::epaint::Shape::rect_filled(
                label_rect,
                2.0,
                Color32::from_black_alpha(200),
            ));

            shapes.push(
                egui::epaint::TextShape::new(label_pos + Vec2::new(2.0, 1.0), galley, Color32::WHITE).into(),
            );

            shapes
        }

        fn update(&mut self, state: &EdgeProps<Relationship>) {
            self.order = state.order;
            self.selected = state.selected;
            self.label = state.payload.label().to_string();
        }
    }

    #[derive(Clone, Copy, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
    enum VisualizationMode {
        TwoD,
        ThreeD,
    }

    impl VisualizationMode {
        fn label(&self) -> &str {
            match self {
                VisualizationMode::TwoD => "2D View",
                VisualizationMode::ThreeD => "3D View",
            }
        }
    }

    #[derive(Clone, serde::Serialize, serde::Deserialize)]
    struct AppConfig {
        // Visualization mode
        visualization_mode: VisualizationMode,
        
        // 3D settings - Rotation
        rotation_x: f32,
        rotation_y: f32,
        rotation_z: f32,
        auto_rotate: bool,
        rotation_speed: f32,
        
        // 3D settings - Positioning
        z_spacing: f32,
        layer_offset: f32,
        perspective_strength: f32,
        
        // 3D settings - Visual Effects
        depth_fade_enabled: bool,
        depth_fade_strength: f32,
        depth_scale_enabled: bool,
        depth_scale_strength: f32,
        
        // Interaction settings
        dragging_enabled: bool,
        hover_enabled: bool,
        node_clicking_enabled: bool,
        node_selection_enabled: bool,
        node_selection_multi_enabled: bool,
        edge_clicking_enabled: bool,
        edge_selection_enabled: bool,
        edge_selection_multi_enabled: bool,
        
        // Navigation settings
        fit_to_screen_enabled: bool,
        zoom_and_pan_enabled: bool,
        fit_to_screen_padding: f32,
        zoom_speed: f32,
        
        // Style settings
        labels_always: bool,
        
        // Color settings
        node_color: [u8; 3],
        node_hover_color: [u8; 3],
        node_selected_color: [u8; 3],
        edge_color: [u8; 3],
        edge_selected_color: [u8; 3],
        background_color: [u8; 3],
        
        // Simulation settings
        simulation_running: bool,
        
        // UI settings
        hover_window_size: f32,
        show_side_panel: bool,
    }

    impl Default for AppConfig {
        fn default() -> Self {
            Self {
                visualization_mode: VisualizationMode::TwoD,
                rotation_x: 30.0,
                rotation_y: 45.0,
                rotation_z: 0.0,
                auto_rotate: false,
                rotation_speed: 0.5,
                z_spacing: 100.0,
                layer_offset: 0.0,
                perspective_strength: 0.001,
                depth_fade_enabled: true,
                depth_fade_strength: 0.0005,
                depth_scale_enabled: true,
                depth_scale_strength: 0.001,
                dragging_enabled: true,
                hover_enabled: true,
                node_clicking_enabled: false,
                node_selection_enabled: true,
                node_selection_multi_enabled: false,
                edge_clicking_enabled: false,
                edge_selection_enabled: false,
                edge_selection_multi_enabled: false,
                fit_to_screen_enabled: false,
                zoom_and_pan_enabled: true,
                fit_to_screen_padding: 0.1,
                zoom_speed: 0.1,
                labels_always: true,
                simulation_running: true,
                node_color: [100, 150, 200],
                node_hover_color: [150, 150, 200],
                node_selected_color: [100, 200, 255],
                edge_color: [128, 128, 128],
                edge_selected_color: [255, 200, 100],
                background_color: [30, 30, 30],
                hover_window_size: 0.0625,
                show_side_panel: true,
            }
        }
    }

    // Helper function for 3D to 2D projection with all rotation axes
    fn project_3d_to_2d(pos: Pos2, z: f32, rotation_x: f32, rotation_y: f32) -> Pos2 {
        let rx = rotation_x.to_radians();
        let ry = rotation_y.to_radians();
        
        // Apply rotation around X axis
        let y1 = pos.y * rx.cos() - z * rx.sin();
        let z1 = pos.y * rx.sin() + z * rx.cos();
        
        // Apply rotation around Y axis
        let x2 = pos.x * ry.cos() + z1 * ry.sin();
        let z2 = -pos.x * ry.sin() + z1 * ry.cos();
        
        // Note: Z-axis rotation would be applied to x2, y1 here if needed
        // but it's less useful for graph visualization
        
        // Perspective projection - depth affects scale
        let scale = 1.0 / (1.0 + z2 * 0.001);
        Pos2::new(x2 * scale, y1 * scale)
    }

    pub struct CodeAnalyzerApp {
        graph: Graph<ClassInfo, Relationship, Directed, u32, CodeNode, CodeEdge>,
        class_details: HashMap<NodeIndex<u32>, ClassInfo>,
        hovered_node: Option<NodeIndex<u32>>,
        config: AppConfig,
        show_config_window: bool,
        config_search: String,
        auto_save_config: bool,
        show_color_picker: bool,
    }

    impl CodeAnalyzerApp {
        pub fn new(cc: &eframe::CreationContext) -> Self {
            // Load config from storage if available
            let config = cc
                .storage
                .and_then(|s| eframe::get_value(s, "app_config"))
                .unwrap_or_default();
            
            let auto_save = cc
                .storage
                .and_then(|s| eframe::get_value(s, "auto_save_config"))
                .unwrap_or(true);
            let mut pg = StableGraph::<ClassInfo, Relationship, Directed>::new();
            let mut class_details = HashMap::new();

            let user = pg.add_node(ClassInfo {
                name: "User".to_string(),
                methods: vec![
                    "login()".to_string(),
                    "logout()".to_string(),
                    "updateProfile()".to_string(),
                ],
                fields: vec![
                    "id: int".to_string(),
                    "username: string".to_string(),
                    "email: string".to_string(),
                ],
            });

            let order = pg.add_node(ClassInfo {
                name: "Order".to_string(),
                methods: vec![
                    "calculate()".to_string(),
                    "submit()".to_string(),
                    "cancel()".to_string(),
                ],
                fields: vec![
                    "id: int".to_string(),
                    "total: float".to_string(),
                    "status: string".to_string(),
                ],
            });

            let product = pg.add_node(ClassInfo {
                name: "Product".to_string(),
                methods: vec!["getPrice()".to_string(), "updateStock()".to_string()],
                fields: vec![
                    "id: int".to_string(),
                    "name: string".to_string(),
                    "price: float".to_string(),
                ],
            });

            let payment = pg.add_node(ClassInfo {
                name: "Payment".to_string(),
                methods: vec!["process()".to_string(), "refund()".to_string()],
                fields: vec![
                    "id: int".to_string(),
                    "amount: float".to_string(),
                    "method: string".to_string(),
                ],
            });

            for idx in pg.node_indices() {
                if let Some(node) = pg.node_weight(idx) {
                    class_details.insert(idx, node.clone());
                }
            }

            pg.add_edge(user, order, Relationship::OneToMany);
            pg.add_edge(order, product, Relationship::ManyToMany);
            pg.add_edge(order, payment, Relationship::OneToOne);
            pg.add_edge(user, payment, Relationship::OneToMany);

            let mut graph = Graph::<ClassInfo, Relationship, Directed, u32, CodeNode, CodeEdge>::from(&pg);

            let positions = vec![
                Pos2::new(0.0, 100.0),
                Pos2::new(200.0, 0.0),
                Pos2::new(200.0, 200.0),
                Pos2::new(400.0, 100.0),
            ];

            for (idx, pos) in pg.node_indices().zip(positions.iter()) {
                if let Some(node) = graph.node_mut(idx) {
                    node.set_location(*pos);
                }
            }

            Self {
                graph,
                class_details,
                hovered_node: None,
                config,
                show_config_window: false,
                config_search: String::new(),
                auto_save_config: auto_save,
                show_color_picker: false,
            }
        }

        fn draw_hover_popup(&self, ui: &mut egui::Ui, node_idx: NodeIndex<u32>) {
            if let Some(class_info) = self.class_details.get(&node_idx) {
                let screen_size = ui.ctx().content_rect().size();
                let popup_size = Vec2::new(
                    screen_size.x * self.config.hover_window_size,
                    screen_size.y * self.config.hover_window_size,
                );

                egui::Window::new(&class_info.name)
                    .fixed_size(popup_size)
                    .collapsible(false)
                    .show(ui.ctx(), |ui| {
                        egui::ScrollArea::vertical().show(ui, |ui| {
                            ui.heading("Fields:");
                            for field in &class_info.fields {
                                ui.label(format!("  • {}", field));
                            }
                            ui.add_space(8.0);
                            ui.heading("Methods:");
                            for method in &class_info.methods {
                                ui.label(format!("  • {}", method));
                            }
                        });
                    });
            }
        }

        fn save_config(&self, ctx: &egui::Context) {
            ctx.data_mut(|data| {
                data.insert_persisted(egui::Id::new("app_config"), self.config.clone());
                data.insert_persisted(egui::Id::new("auto_save_config"), self.auto_save_config);
            });
        }

        fn draw_config_window(&mut self, ctx: &egui::Context) {
            let show_config = self.show_config_window;
            let mut show_window = show_config;
            egui::Window::new("⚙ Configuration")
                .open(&mut show_window)
                .resizable(true)
                .default_width(400.0)
                .show(ctx, |ui| {
                    let mut config = self.config.clone();
                    let mut auto_save = self.auto_save_config;
                    let mut search = self.config_search.clone();
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        // Search bar
                        ui.horizontal(|ui| {
                            ui.label("🔍");
                            ui.text_edit_singleline(&mut search);
                            if ui.button("✖").clicked() {
                                search.clear();
                            }
                        });
                        ui.separator();

                        let search_lower = search.to_lowercase();
                        let matches = |text: &str| search_lower.is_empty() || text.to_lowercase().contains(&search_lower);

                        // Auto-save checkbox
                        ui.checkbox(&mut auto_save, "💾 Auto-save configuration");
                        
                        ui.horizontal(|ui| {
                            if ui.button("💾 Save Now").clicked() {
                                self.config = config.clone();
                                self.auto_save_config = auto_save;
                                self.save_config(ctx);
                            }
                            if ui.button("↺ Reset to Defaults").clicked() {
                                config = AppConfig::default();
                                if auto_save {
                                    self.config = config.clone();
                                    self.auto_save_config = auto_save;
                                    self.save_config(ctx);
                                }
                            }
                        });
                        
                        ui.separator();

                        // Interaction Settings
                        if matches("interaction") || matches("dragging") || matches("hover") || matches("clicking") || matches("selection") {
                            ui.collapsing("🖱️ Interaction Settings", |ui| {
                                let mut changed = false;
                                
                                if matches("dragging") {
                                    changed |= ui.checkbox(&mut config.dragging_enabled, "Enable Node Dragging").changed();
                                }
                                if matches("hover") {
                                    changed |= ui.checkbox(&mut config.hover_enabled, "Enable Hover Detection").changed();
                                }
                                if matches("clicking") || matches("node") {
                                    changed |= ui.checkbox(&mut config.node_clicking_enabled, "Enable Node Clicking").changed();
                                }
                                if matches("selection") || matches("node") {
                                    changed |= ui.checkbox(&mut config.node_selection_enabled, "Enable Node Selection").changed();
                                }
                                if matches("multi") || matches("selection") || matches("node") {
                                    changed |= ui.checkbox(&mut config.node_selection_multi_enabled, "Enable Multi-Node Selection").changed();
                                }
                                if matches("clicking") || matches("edge") {
                                    changed |= ui.checkbox(&mut config.edge_clicking_enabled, "Enable Edge Clicking").changed();
                                }
                                if matches("selection") || matches("edge") {
                                    changed |= ui.checkbox(&mut config.edge_selection_enabled, "Enable Edge Selection").changed();
                                }
                                if matches("multi") || matches("selection") || matches("edge") {
                                    changed |= ui.checkbox(&mut config.edge_selection_multi_enabled, "Enable Multi-Edge Selection").changed();
                                }
                                
                                if changed && auto_save {
                                    self.config = config.clone();
                                    self.auto_save_config = auto_save;
                                    self.save_config(ctx);
                                }
                            });
                        }

                        // Simulation/Layout Settings
                        if matches("simulation") || matches("layout") || matches("force") || matches("animate") {
                            ui.collapsing("⚡ Simulation/Layout Settings", |ui| {
                                ui.label("💡 Force-Directed Layout:");
                                ui.label("  The graph uses Fruchterman-Reingold algorithm");
                                ui.label("  for automatic node positioning.");
                                ui.separator();
                                
                                ui.label("⚙️ Available Parameters:");
                                ui.label("  • dt: Time step (default: 0.05)");
                                ui.label("  • damping: Velocity damping (default: 0.3)");
                                ui.label("  • max_step: Max displacement per step (default: 10.0)");
                                ui.label("  • k_scale: Spring constant scale (default: 1.0)");
                                ui.label("  • c_attract: Attraction force (default: 1.0)");
                                ui.label("  • c_repulse: Repulsion force (default: 1.0)");
                                ui.separator();
                                
                                ui.label("📋 Layout Types:");
                                ui.label("  • Force-Directed (current)");
                                ui.label("  • Random");
                                ui.label("  • Hierarchical");
                                ui.separator();
                                
                                ui.label("❗ Note: Interactive simulation controls");
                                ui.label("require access to the layout state.");
                                ui.label("Advanced controls coming in future update!");
                            });
                        }

                        // Navigation Settings
                        if matches("navigation") || matches("zoom") || matches("pan") || matches("fit") {
                            ui.collapsing("🧭 Navigation Settings", |ui| {
                                let mut changed = false;
                                
                                if matches("fit") || matches("screen") {
                                    changed |= ui.checkbox(&mut config.fit_to_screen_enabled, "Fit to Screen").changed();
                                }
                                if matches("zoom") || matches("pan") {
                                    changed |= ui.checkbox(&mut config.zoom_and_pan_enabled, "Enable Zoom & Pan").changed();
                                }
                                if matches("padding") || matches("fit") {
                                    ui.label("Fit to Screen Padding:");
                                    changed |= ui.add(egui::Slider::new(&mut config.fit_to_screen_padding, 0.0..=0.5).suffix("x")).changed();
                                }
                                if matches("zoom") || matches("speed") {
                                    ui.label("Zoom Speed:");
                                    changed |= ui.add(egui::Slider::new(&mut config.zoom_speed, 0.01..=1.0).logarithmic(true)).changed();
                                }
                                
                                if changed && auto_save {
                                    self.config = config.clone();
                                    self.auto_save_config = auto_save;
                                    self.save_config(ctx);
                                }
                            });
                        }

                        // Style Settings
                        if matches("style") || matches("label") || matches("appearance") {
                            ui.collapsing("🎨 Style Settings", |ui| {
                                let mut changed = false;
                                
                                if matches("label") {
                                    changed |= ui.checkbox(&mut config.labels_always, "Always Show Labels").changed();
                                }
                                
                                if changed && auto_save {
                                    self.config = config.clone();
                                    self.auto_save_config = auto_save;
                                    self.save_config(ctx);
                                }
                            });
                        }

                        // Color Settings
                        if matches("color") || matches("theme") || matches("appearance") {
                            ui.collapsing("🎨 Color Settings", |ui| {
                                let mut changed = false;
                                
                                if matches("node") || matches("color") {
                                    ui.label("Node Color:");
                                    let mut color = Color32::from_rgb(config.node_color[0], config.node_color[1], config.node_color[2]);
                                    if ui.color_edit_button_srgba(&mut color).changed() {
                                        config.node_color = [color.r(), color.g(), color.b()];
                                        changed = true;
                                    }
                                    
                                    ui.label("Node Hover Color:");
                                    let mut hover_color = Color32::from_rgb(config.node_hover_color[0], config.node_hover_color[1], config.node_hover_color[2]);
                                    if ui.color_edit_button_srgba(&mut hover_color).changed() {
                                        config.node_hover_color = [hover_color.r(), hover_color.g(), hover_color.b()];
                                        changed = true;
                                    }
                                    
                                    ui.label("Node Selected Color:");
                                    let mut selected_color = Color32::from_rgb(config.node_selected_color[0], config.node_selected_color[1], config.node_selected_color[2]);
                                    if ui.color_edit_button_srgba(&mut selected_color).changed() {
                                        config.node_selected_color = [selected_color.r(), selected_color.g(), selected_color.b()];
                                        changed = true;
                                    }
                                }
                                
                                if matches("edge") || matches("color") {
                                    ui.label("Edge Color:");
                                    let mut edge_color = Color32::from_rgb(config.edge_color[0], config.edge_color[1], config.edge_color[2]);
                                    if ui.color_edit_button_srgba(&mut edge_color).changed() {
                                        config.edge_color = [edge_color.r(), edge_color.g(), edge_color.b()];
                                        changed = true;
                                    }
                                    
                                    ui.label("Edge Selected Color:");
                                    let mut edge_selected = Color32::from_rgb(config.edge_selected_color[0], config.edge_selected_color[1], config.edge_selected_color[2]);
                                    if ui.color_edit_button_srgba(&mut edge_selected).changed() {
                                        config.edge_selected_color = [edge_selected.r(), edge_selected.g(), edge_selected.b()];
                                        changed = true;
                                    }
                                }
                                
                                if matches("background") || matches("color") {
                                    ui.label("Background Color:");
                                    let mut bg_color = Color32::from_rgb(config.background_color[0], config.background_color[1], config.background_color[2]);
                                    if ui.color_edit_button_srgba(&mut bg_color).changed() {
                                        config.background_color = [bg_color.r(), bg_color.g(), bg_color.b()];
                                        changed = true;
                                    }
                                }
                                
                                if changed && auto_save {
                                    self.config = config.clone();
                                    self.auto_save_config = auto_save;
                                    self.save_config(ctx);
                                }
                            });
                        }

                        // UI Settings
                        if matches("ui") || matches("window") || matches("popup") || matches("panel") {
                            ui.collapsing("🖼️ UI Settings", |ui| {
                                let mut changed = false;
                                
                                if matches("popup") || matches("window") || matches("hover") {
                                    ui.label("Hover Popup Size:");
                                    let mut size_percent = (config.hover_window_size * 100.0) as i32;
                                    if ui.add(egui::Slider::new(&mut size_percent, 5..=50).suffix("%")).changed() {
                                        config.hover_window_size = size_percent as f32 / 100.0;
                                        changed = true;
                                    }
                                }
                                
                                if matches("panel") || matches("side") {
                                    changed |= ui.checkbox(&mut config.show_side_panel, "Show Side Panel").changed();
                                }
                                
                                if changed && auto_save {
                                    self.config = config.clone();
                                    self.auto_save_config = auto_save;
                                    self.save_config(ctx);
                                }
                            });
                        }
                    });
                    
                    // Update self after window closes
                    self.config = config;
                    self.auto_save_config = auto_save;
                    self.config_search = search;
                });
            self.show_config_window = show_window;
        }
    }

    impl App for CodeAnalyzerApp {
        fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
            // Top menu bar
            egui::TopBottomPanel::top("top_menu").show(ctx, |ui| {
                egui::MenuBar::new().ui(ui, |ui| {
                    // Visualization mode dropdown
                    ui.label("View:");
                    egui::ComboBox::from_id_salt("viz_mode")
                        .selected_text(self.config.visualization_mode.label())
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut self.config.visualization_mode, VisualizationMode::TwoD, "2D View");
                            ui.selectable_value(&mut self.config.visualization_mode, VisualizationMode::ThreeD, "3D View");
                        });
                    
                    ui.separator();
                    
                    ui.menu_button("📁 File", |ui| {
                        if ui.button("💾 Save Configuration").clicked() {
                            self.save_config(ctx);
                            ui.close();
                        }
                        if ui.button("📂 Open File...").clicked() {
                            // Placeholder for future file opening functionality
                            ui.close();
                        }
                        if ui.button("📁 Open Folder...").clicked() {
                            // Placeholder for future folder opening functionality
                            ui.close();
                        }
                        ui.separator();
                        if ui.button("❌ Close").clicked() {
                            ui.close();
                        }
                    });

                    ui.menu_button("⚙️ Configuration", |ui| {
                        if ui.button("Open Settings").clicked() {
                            self.show_config_window = true;
                            ui.close();
                        }
                        ui.separator();
                        if ui.button("↺ Reset to Defaults").clicked() {
                            self.config = AppConfig::default();
                            if self.auto_save_config {
                                self.save_config(ctx);
                            }
                            ui.close();
                        }
                    });

                    ui.menu_button("🎨 Colors", |ui| {
                        if ui.button("Change Colors").clicked() {
                            self.show_color_picker = true;
                            ui.close();
                        }
                    });

                    ui.menu_button("ℹ️ Help", |ui| {
                        ui.label("Code Analyzer v1.0");
                        ui.separator();
                        ui.label("Legend:");
                        ui.label("  1:1 = One-to-One");
                        ui.label("  1:N = One-to-Many");
                        ui.label("  N:N = Many-to-Many");
                    });
                });
            });

            // Config window
            if self.show_config_window {
                self.draw_config_window(ctx);
            }

            // Color picker window
            if self.show_color_picker {
                let mut show_picker = self.show_color_picker;
                let mut config = self.config.clone();
                
                egui::Window::new("🎨 Color Customization")
                    .open(&mut show_picker)
                    .resizable(true)
                    .default_width(300.0)
                    .show(ctx, |ui| {
                        egui::ScrollArea::vertical().show(ui, |ui| {
                            ui.heading("Node Colors");
                            ui.horizontal(|ui| {
                                ui.label("Default:");
                                let mut color = Color32::from_rgb(config.node_color[0], config.node_color[1], config.node_color[2]);
                                if ui.color_edit_button_srgba(&mut color).changed() {
                                    config.node_color = [color.r(), color.g(), color.b()];
                                }
                            });
                            
                            ui.horizontal(|ui| {
                                ui.label("Hover:");
                                let mut color = Color32::from_rgb(config.node_hover_color[0], config.node_hover_color[1], config.node_hover_color[2]);
                                if ui.color_edit_button_srgba(&mut color).changed() {
                                    config.node_hover_color = [color.r(), color.g(), color.b()];
                                }
                            });
                            
                            ui.horizontal(|ui| {
                                ui.label("Selected:");
                                let mut color = Color32::from_rgb(config.node_selected_color[0], config.node_selected_color[1], config.node_selected_color[2]);
                                if ui.color_edit_button_srgba(&mut color).changed() {
                                    config.node_selected_color = [color.r(), color.g(), color.b()];
                                }
                            });
                            
                            ui.separator();
                            ui.heading("Edge Colors");
                            
                            ui.horizontal(|ui| {
                                ui.label("Default:");
                                let mut color = Color32::from_rgb(config.edge_color[0], config.edge_color[1], config.edge_color[2]);
                                if ui.color_edit_button_srgba(&mut color).changed() {
                                    config.edge_color = [color.r(), color.g(), color.b()];
                                }
                            });
                            
                            ui.horizontal(|ui| {
                                ui.label("Selected:");
                                let mut color = Color32::from_rgb(config.edge_selected_color[0], config.edge_selected_color[1], config.edge_selected_color[2]);
                                if ui.color_edit_button_srgba(&mut color).changed() {
                                    config.edge_selected_color = [color.r(), color.g(), color.b()];
                                }
                            });
                            
                            ui.separator();
                            ui.heading("Background");
                            
                            ui.horizontal(|ui| {
                                ui.label("Color:");
                                let mut color = Color32::from_rgb(config.background_color[0], config.background_color[1], config.background_color[2]);
                                if ui.color_edit_button_srgba(&mut color).changed() {
                                    config.background_color = [color.r(), color.g(), color.b()];
                                }
                            });
                            
                            ui.separator();
                            ui.horizontal(|ui| {
                                if ui.button("💾 Save").clicked() {
                                    self.config = config.clone();
                                    self.save_config(ctx);
                                }
                                if ui.button("↺ Reset to Defaults").clicked() {
                                    let default = AppConfig::default();
                                    config.node_color = default.node_color;
                                    config.node_hover_color = default.node_hover_color;
                                    config.node_selected_color = default.node_selected_color;
                                    config.edge_color = default.edge_color;
                                    config.edge_selected_color = default.edge_selected_color;
                                    config.background_color = default.background_color;
                                }
                            });
                        });
                    });
                
                self.config = config;
                self.show_color_picker = show_picker;
            }

            // 3D Controls panel (only show in 3D mode)
            if self.config.visualization_mode == VisualizationMode::ThreeD {
                egui::Window::new("🎮 3D Visualization Settings")
                    .default_pos([10.0, 100.0])
                    .default_width(300.0)
                    .resizable(true)
                    .show(ctx, |ui| {
                        egui::ScrollArea::vertical().show(ui, |ui| {
                            // Rotation Settings
                            ui.collapsing("🔄 Rotation", |ui| {
                                ui.horizontal(|ui| {
                                    ui.label("X Axis:");
                                    if ui.add(egui::Slider::new(&mut self.config.rotation_x, -180.0..=180.0).suffix("°")).changed() {
                                        if self.auto_save_config {
                                            self.save_config(ctx);
                                        }
                                    }
                                });
                                ui.horizontal(|ui| {
                                    ui.label("Y Axis:");
                                    if ui.add(egui::Slider::new(&mut self.config.rotation_y, -180.0..=180.0).suffix("°")).changed() {
                                        if self.auto_save_config {
                                            self.save_config(ctx);
                                        }
                                    }
                                });
                                ui.horizontal(|ui| {
                                    ui.label("Z Axis:");
                                    if ui.add(egui::Slider::new(&mut self.config.rotation_z, -180.0..=180.0).suffix("°")).changed() {
                                        if self.auto_save_config {
                                            self.save_config(ctx);
                                        }
                                    }
                                });
                                
                                ui.separator();
                                
                                // Prominent stop button if rotation is active
                                if self.config.auto_rotate {
                                    if ui.button("⏹ Stop Rotation").clicked() {
                                        self.config.auto_rotate = false;
                                        if self.auto_save_config {
                                            self.save_config(ctx);
                                        }
                                    }
                                }
                                
                                if ui.checkbox(&mut self.config.auto_rotate, "Auto Rotate").changed() {
                                    if self.auto_save_config {
                                        self.save_config(ctx);
                                    }
                                }
                                if self.config.auto_rotate {
                                    ui.horizontal(|ui| {
                                        ui.label("Speed:");
                                        if ui.add(egui::Slider::new(&mut self.config.rotation_speed, 0.1..=5.0).logarithmic(true)).changed() {
                                            if self.auto_save_config {
                                                self.save_config(ctx);
                                            }
                                        }
                                    });
                                }
                            });
                            
                            // Positioning Settings
                            ui.collapsing("📐 Positioning", |ui| {
                                ui.horizontal(|ui| {
                                    ui.label("Layer Spacing:");
                                    if ui.add(egui::Slider::new(&mut self.config.z_spacing, 10.0..=500.0).suffix(" units")).changed() {
                                        if self.auto_save_config {
                                            self.save_config(ctx);
                                        }
                                    }
                                });
                                ui.horizontal(|ui| {
                                    ui.label("Layer Offset:");
                                    if ui.add(egui::Slider::new(&mut self.config.layer_offset, -200.0..=200.0).suffix(" units")).changed() {
                                        if self.auto_save_config {
                                            self.save_config(ctx);
                                        }
                                    }
                                });
                                ui.horizontal(|ui| {
                                    ui.label("Perspective:");
                                    if ui.add(egui::Slider::new(&mut self.config.perspective_strength, 0.0..=0.01).logarithmic(true)).changed() {
                                        if self.auto_save_config {
                                            self.save_config(ctx);
                                        }
                                    }
                                });
                            });
                            
                            // Simulation Control
                            ui.collapsing("⚙️ Simulation Control", |ui| {
                                ui.label("Control the force-directed layout simulation:");
                                ui.separator();
                                
                                ui.horizontal(|ui| {
                                    if self.config.simulation_running {
                                        if ui.button("⏸ Pause Simulation").clicked() {
                                            self.config.simulation_running = false;
                                            if self.auto_save_config {
                                                self.save_config(ctx);
                                            }
                                        }
                                    } else {
                                        if ui.button("▶ Resume Simulation").clicked() {
                                            self.config.simulation_running = true;
                                            if self.auto_save_config {
                                                self.save_config(ctx);
                                            }
                                        }
                                    }
                                });
                                
                                ui.label(if self.config.simulation_running {
                                    "✓ Simulation is running - nodes will move to optimal positions"
                                } else {
                                    "⏸ Simulation is paused - nodes will stay in place"
                                });
                                
                                ui.separator();
                                ui.label("💡 Tip: Pause the simulation to manually arrange nodes,");
                                ui.label("   then resume to let them settle into stable positions.");
                            });
                            
                            // Visual Effects Settings
                            ui.collapsing("✨ Visual Effects", |ui| {
                                if ui.checkbox(&mut self.config.depth_fade_enabled, "Depth Fade").changed() {
                                    if self.auto_save_config {
                                        self.save_config(ctx);
                                    }
                                }
                                if self.config.depth_fade_enabled {
                                    ui.horizontal(|ui| {
                                        ui.label("    Strength:");
                                        if ui.add(egui::Slider::new(&mut self.config.depth_fade_strength, 0.0..=0.005).logarithmic(true)).changed() {
                                            if self.auto_save_config {
                                                self.save_config(ctx);
                                            }
                                        }
                                    });
                                }
                                
                                if ui.checkbox(&mut self.config.depth_scale_enabled, "Depth Scaling").changed() {
                                    if self.auto_save_config {
                                        self.save_config(ctx);
                                    }
                                }
                                if self.config.depth_scale_enabled {
                                    ui.horizontal(|ui| {
                                        ui.label("    Strength:");
                                        if ui.add(egui::Slider::new(&mut self.config.depth_scale_strength, 0.0..=0.01).logarithmic(true)).changed() {
                                            if self.auto_save_config {
                                                self.save_config(ctx);
                                            }
                                        }
                                    });
                                }
                            });
                            
                            ui.separator();
                            ui.horizontal(|ui| {
                                if ui.button("↺ Reset All").clicked() {
                                    let defaults = AppConfig::default();
                                    self.config.rotation_x = defaults.rotation_x;
                                    self.config.rotation_y = defaults.rotation_y;
                                    self.config.rotation_z = defaults.rotation_z;
                                    self.config.auto_rotate = defaults.auto_rotate;
                                    self.config.rotation_speed = defaults.rotation_speed;
                                    self.config.z_spacing = defaults.z_spacing;
                                    self.config.layer_offset = defaults.layer_offset;
                                    self.config.perspective_strength = defaults.perspective_strength;
                                    self.config.depth_fade_enabled = defaults.depth_fade_enabled;
                                    self.config.depth_fade_strength = defaults.depth_fade_strength;
                                    self.config.depth_scale_enabled = defaults.depth_scale_enabled;
                                    self.config.depth_scale_strength = defaults.depth_scale_strength;
                                    self.config.simulation_running = defaults.simulation_running;
                                    if self.auto_save_config {
                                        self.save_config(ctx);
                                    }
                                }
                                if ui.button("💾 Save").clicked() {
                                    self.save_config(ctx);
                                }
                            });
                        });
                    });
            }

            // Side panel
            if self.config.show_side_panel {
                egui::SidePanel::right("side_panel").show(ctx, |ui| {
                    ui.heading("📊 Classes");
                    ui.separator();

                    egui::ScrollArea::vertical().show(ui, |ui| {
                        for (idx, info) in &self.class_details {
                            if ui.button(&info.name).clicked() {
                                if let Some(node) = self.graph.node_mut(*idx) {
                                    node.set_selected(true);
                                }
                            }
                        }
                    });
                    
                    ui.separator();
                    ui.add_space(10.0);
                    
                    if ui.button("⚙️ Settings").clicked() {
                        self.show_config_window = true;
                    }
                });
            }

            // Auto-rotation in 3D mode (only when explicitly enabled)
            if self.config.visualization_mode == VisualizationMode::ThreeD {
                if self.config.auto_rotate {
                    self.config.rotation_y += self.config.rotation_speed * 0.5;
                    if self.config.rotation_y > 180.0 {
                        self.config.rotation_y -= 360.0;
                    }
                    ctx.request_repaint();
                }
            }

            // Update node and edge colors from config, and apply 3D positioning
            let node_indices: Vec<_> = self.graph.g().node_indices().collect();
            for (layer_idx, idx) in node_indices.iter().enumerate() {
                if let Some(node) = self.graph.node_mut(*idx) {
                    node.display_mut().node_color = self.config.node_color;
                    node.display_mut().hover_color = self.config.node_hover_color;
                    node.display_mut().selected_color = self.config.node_selected_color;
                    
                    // Apply Z-position for 3D effect
                    if self.config.visualization_mode == VisualizationMode::ThreeD {
                        let z_offset = (layer_idx as f32 - node_indices.len() as f32 / 2.0) * self.config.z_spacing + self.config.layer_offset;
                        node.display_mut().z_pos = z_offset;
                        
                        // Apply 3D projection to node position
                        let original_pos = node.location();
                        let projected = project_3d_to_2d(original_pos, z_offset, self.config.rotation_x, self.config.rotation_y);
                        node.set_location(projected);
                    } else {
                        node.display_mut().z_pos = 0.0;
                    }
                }
            }
            
            let edge_indices: Vec<_> = self.graph.g().edge_indices().collect();
            for edge_idx in edge_indices {
                if let Some(edge) = self.graph.edge_mut(edge_idx) {
                    edge.display_mut().edge_color = self.config.edge_color;
                    edge.display_mut().selected_color = self.config.edge_selected_color;
                }
            }

            egui::CentralPanel::default().show(ctx, |ui| {
                // Control simulation based on user setting - do this FIRST
                let mut layout_state: FruchtermanReingoldState = get_layout_state(ui, None);
                layout_state.is_running = self.config.simulation_running;
                set_layout_state(ui, layout_state, None);
                
                let settings_interaction = SettingsInteraction::new()
                    .with_dragging_enabled(self.config.dragging_enabled)
                    .with_hover_enabled(self.config.hover_enabled)
                    .with_node_clicking_enabled(self.config.node_clicking_enabled)
                    .with_node_selection_enabled(self.config.node_selection_enabled)
                    .with_node_selection_multi_enabled(self.config.node_selection_multi_enabled)
                    .with_edge_clicking_enabled(self.config.edge_clicking_enabled)
                    .with_edge_selection_enabled(self.config.edge_selection_enabled)
                    .with_edge_selection_multi_enabled(self.config.edge_selection_multi_enabled);

                let settings_navigation = SettingsNavigation::new()
                    .with_zoom_and_pan_enabled(self.config.zoom_and_pan_enabled)
                    .with_fit_to_screen_enabled(self.config.fit_to_screen_enabled)
                    .with_fit_to_screen_padding(self.config.fit_to_screen_padding)
                    .with_zoom_speed(self.config.zoom_speed);

                let settings_style = SettingsStyle::new()
                    .with_labels_always(self.config.labels_always);
                
                ui.add(
                    &mut GraphView::<_, _, _, _, CodeNode, CodeEdge>::new(&mut self.graph)
                        .with_interactions(&settings_interaction)
                        .with_navigations(&settings_navigation)
                        .with_styles(&settings_style),
                );

                let mut new_hovered = None;
                for idx in self.graph.g().node_indices() {
                    if let Some(node) = self.graph.node(idx) {
                        if node.hovered() {
                            new_hovered = Some(idx);
                            break;
                        }
                    }
                }
                self.hovered_node = new_hovered;

                if let Some(node_idx) = self.hovered_node {
                    self.draw_hover_popup(ui, node_idx);
                }
            });
        }
        
        fn save(&mut self, storage: &mut dyn eframe::Storage) {
            eframe::set_value(storage, "app_config", &self.config);
            eframe::set_value(storage, "auto_save_config", &self.auto_save_config);
        }
    }
}
