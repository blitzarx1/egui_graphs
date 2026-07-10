use edge::RainbowEdgeShape;
use eframe::{run_native, App, CreationContext};
use egui_graphs::{generate_simple_digraph, DefaultGraphView, DefaultNodeShape, Graph};
use petgraph::{csr::DefaultIx, Directed};

pub struct RainbowEdgesApp {
    g: Graph<(), (), Directed, DefaultIx, DefaultNodeShape, RainbowEdgeShape>,
}

impl RainbowEdgesApp {
    fn new(_: &CreationContext<'_>) -> Self {
        let g = generate_simple_digraph();
        Self { g: Graph::from(&g) }
    }
}

impl App for RainbowEdgesApp {
    fn ui(&mut self, ui: &mut egui::Ui, _: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ui, |ui| {
            DefaultGraphView::new()
                .with_interactions(
                    &egui_graphs::SettingsInteraction::default().with_dragging_enabled(true),
                )
                .show(ui, &mut self.g);
        });
    }
}

fn main() {
    let native_options = eframe::NativeOptions::default();
    run_native(
        "rainbow_edges",
        native_options,
        Box::new(|cc| Ok(Box::new(RainbowEdgesApp::new(cc)))),
    )
    .unwrap();
}

mod edge {
    use egui::{Color32, Pos2, Stroke, Vec2};
    use egui_graphs::{
        DefaultEdgeShape, DisplayEdge, DisplayNode, DrawContext, EdgeProps, EdgeShapeBuilder,
        EdgeShapeProps, Node, TipProps,
    };
    use petgraph::{stable_graph::IndexType, EdgeType};

    const TIP_ANGLE: f32 = std::f32::consts::TAU / 30.;
    const TIP_SIZE: f32 = 15.;
    const COLORS: [Color32; 7] = [
        Color32::RED,
        Color32::from_rgb(255, 102, 0),
        Color32::YELLOW,
        Color32::GREEN,
        Color32::from_rgb(2, 216, 233),
        Color32::BLUE,
        Color32::from_rgb(91, 10, 145),
    ];

    #[derive(Clone)]
    pub struct RainbowEdgeShape {
        default_impl: DefaultEdgeShape,
    }

    impl<E: Clone> From<EdgeProps<E>> for RainbowEdgeShape {
        fn from(props: EdgeProps<E>) -> Self {
            Self {
                default_impl: DefaultEdgeShape::from(props),
            }
        }
    }

    impl<N: Clone, E: Clone, Ty: EdgeType, Ix: IndexType, D: DisplayNode<N, E, Ty, Ix>>
        DisplayEdge<N, E, Ty, Ix, D> for RainbowEdgeShape
    {
        fn shapes(
            &mut self,
            start: &Node<N, E, Ty, Ix, D>,
            end: &Node<N, E, Ty, Ix, D>,
            ctx: &DrawContext,
        ) -> Vec<egui::Shape> {
            let mut res = vec![];
            let (start, end) = (start.location(), end.location());
            let (x_dist, y_dist) = (end.x - start.x, end.y - start.y);
            let (dx, dy) = (x_dist / COLORS.len() as f32, y_dist / COLORS.len() as f32);
            let d_vec = Vec2::new(dx, dy);

            let style = ctx.ctx.global_style();

            for (i, color) in COLORS.iter().enumerate() {
                let bounds = (
                    start + i as f32 * d_vec,
                    end - (COLORS.len() - i - 1) as f32 * d_vec,
                );
                let stroke = ctx.style.resolve_edge_stroke(
                    self.default_impl.selected,
                    self.default_impl.order,
                    Stroke::new(self.default_impl.width, *color),
                    &style,
                );
                let mut builder =
                    EdgeShapeBuilder::new(EdgeShapeProps::Straight { bounds }, stroke)
                        .with_scaler(ctx.meta);

                if ctx.is_directed && i == COLORS.len() - 1 {
                    builder = builder.with_tip(TipProps {
                        size: TIP_SIZE,
                        angle: TIP_ANGLE,
                    });
                }
                res.extend(builder.build().into_shapes());
            }

            res
        }

        fn update(&mut self, props: &egui_graphs::EdgeProps<E>) {
            self.default_impl.order = props.order;
            self.default_impl.selected = props.selected;
            self.default_impl.label_text.clone_from(&props.label);
        }

        fn is_inside(
            &self,
            start: &Node<N, E, Ty, Ix, D>,
            end: &Node<N, E, Ty, Ix, D>,
            pos: Pos2,
        ) -> bool {
            self.default_impl.is_inside(start, end, pos)
        }
    }
}
