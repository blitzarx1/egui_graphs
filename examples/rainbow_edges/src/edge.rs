use egui::{Color32, Pos2, Shape, Stroke, Vec2};
use egui_graphs::{DefaultEdgeShape, DisplayEdge, DisplayNode, DrawContext, EdgeProps, Node};
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
        let (start, end) = (start.location().unwrap(), end.location().unwrap());
        let (x_dist, y_dist) = (end.x - start.x, end.y - start.y);
        let (dx, dy) = (x_dist / COLORS.len() as f32, y_dist / COLORS.len() as f32);
        let d_vec = Vec2::new(dx, dy);

        let mut stroke = Stroke::default();
        let mut points_line;

        for (i, color) in COLORS.iter().enumerate() {
            stroke = Stroke::new(self.default_impl.width, *color);
            points_line = vec![
                start + i as f32 * d_vec,
                end - (COLORS.len() - i - 1) as f32 * d_vec,
            ];

            stroke.width = ctx.meta.canvas_to_screen_size(stroke.width);
            points_line = points_line
                .iter()
                .map(|p| ctx.meta.canvas_to_screen_pos(*p))
                .collect();
            res.push(Shape::line_segment(
                [points_line[0], points_line[1]],
                stroke,
            ));
        }

        let tip_dir = (end - start).normalized();

        let arrow_tip_dir_1 = rotate_vector(tip_dir, TIP_ANGLE) * TIP_SIZE;
        let arrow_tip_dir_2 = rotate_vector(tip_dir, -TIP_ANGLE) * TIP_SIZE;

        let tip_start_1 = end - arrow_tip_dir_1;
        let tip_start_2 = end - arrow_tip_dir_2;

        let mut points_tip = vec![end, tip_start_1, tip_start_2];

        points_tip = points_tip
            .iter()
            .map(|p| ctx.meta.canvas_to_screen_pos(*p))
            .collect();

        res.push(Shape::convex_polygon(
            points_tip,
            stroke.color,
            Stroke::default(),
        ));

        res
    }

    fn update(&mut self, _: &egui_graphs::EdgeProps<E>) {}

    fn is_inside(
        &self,
        start: &Node<N, E, Ty, Ix, D>,
        end: &Node<N, E, Ty, Ix, D>,
        pos: Pos2,
    ) -> bool {
        self.default_impl.is_inside(start, end, pos)
    }
}

/// rotates vector by angle
fn rotate_vector(vec: Vec2, angle: f32) -> Vec2 {
    let cos = angle.cos();
    let sin = angle.sin();
    Vec2::new(cos * vec.x - sin * vec.y, sin * vec.x + cos * vec.y)
}
