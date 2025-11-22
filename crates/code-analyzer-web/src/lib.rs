use eframe::wasm_bindgen::{self, prelude::*};

#[wasm_bindgen]
pub struct WebHandle {
    runner: eframe::WebRunner,
}

#[wasm_bindgen]
impl WebHandle {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            runner: eframe::WebRunner::new(),
        }
    }

    #[wasm_bindgen]
    pub async fn start(&self, canvas_id: &str) -> Result<(), wasm_bindgen::JsValue> {
        self.runner
            .start(
                canvas_id,
                eframe::WebOptions::default(),
                Box::new(|cc| Ok(Box::new(code_analyzer::CodeAnalyzerApp::new(cc)))),
            )
            .await
    }
}

mod code_analyzer {
    use eframe::App;
    use egui::{Color32, FontFamily, FontId, Pos2, Rect, Shape, Stroke, Vec2};
    use egui_graphs::{
        DisplayEdge, DisplayNode, DrawContext, EdgeProps, Graph, GraphView, Node, NodeProps,
        SettingsInteraction, SettingsNavigation,
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
        selected: bool,
        dragged: bool,
        hovered: bool,
        class_name: String,
        radius: f32,
    }

    impl From<NodeProps<ClassInfo>> for CodeNode {
        fn from(node_props: NodeProps<ClassInfo>) -> Self {
            Self {
                pos: node_props.location(),
                selected: node_props.selected,
                dragged: node_props.dragged,
                hovered: node_props.hovered,
                class_name: node_props.payload.name.clone(),
                radius: 30.0,
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

            let color = if self.selected {
                Color32::from_rgb(100, 200, 255)
            } else if self.hovered {
                Color32::from_rgb(150, 150, 200)
            } else {
                Color32::from_rgb(100, 150, 200)
            };

            let stroke = if self.selected {
                Stroke::new(2.0, Color32::WHITE)
            } else {
                Stroke::new(1.0, Color32::GRAY)
            };

            shapes.push(
                egui::epaint::CircleShape {
                    center: screen_pos,
                    radius: screen_radius,
                    fill: color,
                    stroke,
                }
                .into(),
            );

            let font_size = (screen_radius * 0.4).max(8.0).min(16.0);
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
    }

    impl From<EdgeProps<Relationship>> for CodeEdge {
        fn from(edge_props: EdgeProps<Relationship>) -> Self {
            Self {
                order: edge_props.order,
                selected: edge_props.selected,
                label: edge_props.payload.label().to_string(),
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
                Color32::from_rgb(255, 200, 100)
            } else {
                Color32::GRAY
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

    pub struct CodeAnalyzerApp {
        graph: Graph<ClassInfo, Relationship, Directed, u32, CodeNode, CodeEdge>,
        class_details: HashMap<NodeIndex<u32>, ClassInfo>,
        hover_window_size: f32,
        hovered_node: Option<NodeIndex<u32>>,
    }

    impl CodeAnalyzerApp {
        pub fn new(_cc: &eframe::CreationContext) -> Self {
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
                hover_window_size: 0.0625,
                hovered_node: None,
            }
        }

        fn draw_hover_popup(&self, ui: &mut egui::Ui, node_idx: NodeIndex<u32>) {
            if let Some(class_info) = self.class_details.get(&node_idx) {
                let screen_size = ui.ctx().content_rect().size();
                let popup_size = Vec2::new(
                    screen_size.x * self.hover_window_size,
                    screen_size.y * self.hover_window_size,
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
    }

    impl App for CodeAnalyzerApp {
        fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
            egui::SidePanel::right("settings").show(ctx, |ui| {
                ui.heading("Code Analyzer Settings");
                ui.separator();

                ui.label("Popup Window Size:");
                let mut size_percent = (self.hover_window_size * 100.0) as i32;
                if ui
                    .add(egui::Slider::new(&mut size_percent, 5..=50).suffix("%"))
                    .changed()
                {
                    self.hover_window_size = size_percent as f32 / 100.0;
                }

                ui.add_space(10.0);
                ui.label("Legend:");
                ui.label("🔵 Hover over nodes to see details");
                ui.label("1:1 = One-to-One");
                ui.label("1:N = One-to-Many");
                ui.label("N:N = Many-to-Many");

                ui.add_space(10.0);
                ui.heading("Classes:");
                for (idx, info) in &self.class_details {
                    if ui.button(&info.name).clicked() {
                        if let Some(node) = self.graph.node_mut(*idx) {
                            node.set_selected(true);
                        }
                    }
                }
            });

            egui::CentralPanel::default().show(ctx, |ui| {
                let settings_interaction = SettingsInteraction::new()
                    .with_dragging_enabled(true)
                    .with_hover_enabled(true);

                let settings_navigation = SettingsNavigation::new()
                    .with_zoom_and_pan_enabled(true)
                    .with_fit_to_screen_enabled(false);

                ui.add(
                    &mut GraphView::<_, _, _, _, CodeNode, CodeEdge>::new(&mut self.graph)
                        .with_interactions(&settings_interaction)
                        .with_navigations(&settings_navigation),
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
    }
}
