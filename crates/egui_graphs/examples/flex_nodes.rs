use eframe::{run_native, App, CreationContext};
use egui::{CentralPanel, Context, SidePanel, TextEdit};
use egui_graphs::{generate_simple_digraph, Graph, GraphView, SettingsInteraction};
use node::NodeShapeFlex;
use petgraph::{
    stable_graph::{DefaultIx, EdgeIndex, NodeIndex},
    Directed,
};

pub struct FlexNodesApp {
    g: Graph<(), (), Directed, DefaultIx, NodeShapeFlex>,
    label_input: String,
    selected_node: Option<NodeIndex>,
    selected_edge: Option<EdgeIndex>,
}

impl FlexNodesApp {
    fn new(_: &CreationContext<'_>) -> Self {
        let g = generate_simple_digraph();
        Self {
            g: Graph::from(&g),
            label_input: String::default(),
            selected_node: Option::default(),
            selected_edge: Option::default(),
        }
    }

    fn read_data(&mut self) {
        if !self.g.selected_nodes().is_empty() {
            let idx = self.g.selected_nodes().first().unwrap();
            self.selected_node = Some(*idx);
            self.selected_edge = None;
            self.label_input = self.g.node(*idx).unwrap().label();
        }
        if !self.g.selected_edges().is_empty() {
            let idx = self.g.selected_edges().first().unwrap();
            self.selected_edge = Some(*idx);
            self.selected_node = None;
            self.label_input = self.g.edge(*idx).unwrap().label();
        }
    }

    fn render(&mut self, ctx: &Context) {
        SidePanel::right("right_panel").show(ctx, |ui| {
            ui.label("Select a node to change its label");
            ui.add_enabled_ui(
                self.selected_node.is_some() || self.selected_edge.is_some(),
                |ui| {
                    TextEdit::singleline(&mut self.label_input)
                        .hint_text("select node or edge")
                        .show(ui)
                },
            );
            if ui.button("reset").clicked() {
                self.reset(ui);
            }
        });
        CentralPanel::default().show(ctx, |ui| {
            let widget = &mut GraphView::<_, _, _, _, _, _>::new(&mut self.g).with_interactions(
                &SettingsInteraction::default().with_node_selection_enabled(true),
            );
            ui.add(widget);
        });
    }

    fn update_data(&mut self) {
        if self.selected_node.is_none() && self.selected_edge.is_none() {
            return;
        }

        if self.selected_node.is_some() {
            let idx = self.selected_node.unwrap();
            if idx.index().to_string() == self.label_input {
                return;
            }

            self.g
                .node_mut(idx)
                .unwrap()
                .set_label(self.label_input.clone());
        }

        if self.selected_edge.is_some() {
            let idx = self.selected_edge.unwrap();
            if idx.index().to_string() == self.label_input {
                return;
            }

            self.g
                .edge_mut(idx)
                .unwrap()
                .set_label(self.label_input.clone());
        }
    }

    fn reset(&mut self, ui: &mut egui::Ui) {
        let g = generate_simple_digraph();
        *self = Self {
            g: Graph::from(&g),
            label_input: String::default(),
            selected_node: Option::default(),
            selected_edge: Option::default(),
        };

        egui_graphs::reset::<egui_graphs::LayoutStateRandom>(ui, None);
    }
}

impl App for FlexNodesApp {
    fn update(&mut self, ctx: &Context, _: &mut eframe::Frame) {
        self.read_data();
        self.render(ctx);
        self.update_data();
    }
}

fn main() {
    let native_options = eframe::NativeOptions::default();
    run_native(
        "flex_nodes",
        native_options,
        Box::new(|cc| Ok(Box::new(FlexNodesApp::new(cc)))),
    )
    .unwrap();
}

mod node {
    use egui::{epaint::TextShape, Color32, FontFamily, FontId, Pos2, Rect, Shape, Stroke, Vec2};
    use egui_graphs::{DisplayNode, NodeProps};
    use petgraph::{stable_graph::IndexType, EdgeType};

    #[derive(Clone)]
    pub struct NodeShapeFlex {
        label: String,
        loc: Pos2,

        size_x: f32,
        size_y: f32,
    }

    impl<N: Clone> From<NodeProps<N>> for NodeShapeFlex {
        fn from(node_props: NodeProps<N>) -> Self {
            Self {
                label: node_props.label.clone(),
                loc: node_props.location(),

                size_x: 0.,
                size_y: 0.,
            }
        }
    }

    impl<N: Clone, E: Clone, Ty: EdgeType, Ix: IndexType> DisplayNode<N, E, Ty, Ix> for NodeShapeFlex {
        fn is_inside(&self, pos: Pos2) -> bool {
            let rect = Rect::from_center_size(self.loc, Vec2::new(self.size_x, self.size_y));

            rect.contains(pos)
        }

        fn closest_boundary_point(&self, dir: Vec2) -> Pos2 {
            find_intersection(self.loc, self.size_x / 2., self.size_y / 2., dir)
        }

        fn shapes(&mut self, ctx: &egui_graphs::DrawContext) -> Vec<egui::Shape> {
            // find node center location on the screen coordinates
            let center = ctx.meta.canvas_to_screen_pos(self.loc);
            let color = ctx.ctx.style().visuals.text_color();

            // create label
            let galley = ctx.ctx.fonts_mut(|f| {
                f.layout_no_wrap(
                    self.label.clone(),
                    FontId::new(ctx.meta.canvas_to_screen_size(10.), FontFamily::Monospace),
                    color,
                )
            });

            // we need to offset label by half its size to place it in the center of the rect
            let offset = Vec2::new(-galley.size().x / 2., -galley.size().y / 2.);

            // create the shape and add it to the layers
            let shape_label = TextShape::new(center + offset, galley, color);

            let rect = shape_label.visual_bounding_rect();
            let points = rect_to_points(rect);
            let shape_rect =
                Shape::convex_polygon(points, Color32::default(), Stroke::new(1., color));

            // update self size
            self.size_x = rect.size().x;
            self.size_y = rect.size().y;

            vec![shape_rect, shape_label.into()]
        }

        fn update(&mut self, state: &NodeProps<N>) {
            self.label.clone_from(&state.label);
            self.loc = state.location();
        }
    }

    fn find_intersection(center: Pos2, size_x: f32, size_y: f32, direction: Vec2) -> Pos2 {
        if (direction.x.abs() * size_y) > (direction.y.abs() * size_x) {
            // intersects left or right side
            let x = if direction.x > 0.0 {
                center.x + size_x / 2.0
            } else {
                center.x - size_x / 2.0
            };
            let y = center.y + direction.y / direction.x * (x - center.x);
            Pos2::new(x, y)
        } else {
            // intersects top or bottom side
            let y = if direction.y > 0.0 {
                center.y + size_y / 2.0
            } else {
                center.y - size_y / 2.0
            };
            let x = center.x + direction.x / direction.y * (y - center.y);
            Pos2::new(x, y)
        }
    }

    fn rect_to_points(rect: Rect) -> Vec<Pos2> {
        let top_left = rect.min;
        let bottom_right = rect.max;
        let top_right = Pos2::new(bottom_right.x, top_left.y);
        let bottom_left = Pos2::new(top_left.x, bottom_right.y);

        vec![top_left, top_right, bottom_right, bottom_left]
    }
}
