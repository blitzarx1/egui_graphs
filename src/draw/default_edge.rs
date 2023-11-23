use std::f32::consts::PI;

use egui::{
    epaint::{CubicBezierShape, QuadraticBezierShape},
    Color32, Pos2, Shape, Stroke, Vec2,
};
use petgraph::{matrix_graph::Nullable, stable_graph::IndexType, EdgeType};

use crate::{draw::DrawContext, elements::EdgeProps, DisplayNode, Node};

use super::DisplayEdge;

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug)]
pub struct DefaultEdgeShape {
    pub order: usize,
    pub selected: bool,

    pub width: f32,
    pub tip_size: f32,
    pub tip_angle: f32,
    pub curve_size: f32,
    pub loop_size: f32,
}

impl<E: Clone> From<EdgeProps<E>> for DefaultEdgeShape {
    fn from(edge: EdgeProps<E>) -> Self {
        Self {
            order: edge.order,
            selected: edge.selected,

            width: 2.,
            tip_size: 15.,
            tip_angle: std::f32::consts::TAU / 30.,
            curve_size: 20.,
            loop_size: 3.,
        }
    }
}

impl<N: Clone, E: Clone, Ty: EdgeType, Ix: IndexType, D: DisplayNode<N, E, Ty, Ix>>
    DisplayEdge<N, E, Ty, Ix, D> for DefaultEdgeShape
{
    fn is_inside(
        &self,
        start: &Node<N, E, Ty, Ix, D>,
        end: &Node<N, E, Ty, Ix, D>,
        pos: egui::Pos2,
    ) -> bool {
        if start.id() == end.id() {
            return is_inside_loop(start, self, pos);
        }

        let pos_start = start.location();
        let pos_end = end.location();

        if self.order == 0 {
            return is_inside_line(pos_start, pos_end, pos, self);
        }

        is_inside_curve(start, end, self, pos)
    }

    fn shapes(
        &mut self,
        start: &Node<N, E, Ty, Ix, D>,
        end: &Node<N, E, Ty, Ix, D>,
        ctx: &DrawContext,
    ) -> Vec<egui::Shape> {
        let style = match self.selected {
            true => ctx.ctx.style().visuals.widgets.active,
            false => ctx.ctx.style().visuals.widgets.inactive,
        };
        let color = style.fg_stroke.color;

        if start.id() == end.id() {
            // draw loop
            let node_size = node_size(start);
            let stroke = Stroke::new(self.width * ctx.meta.zoom, color);
            return vec![shape_looped(
                ctx.meta.canvas_to_screen_size(node_size),
                ctx.meta.canvas_to_screen_pos(start.location()),
                stroke,
                self,
            )
            .into()];
        }

        let dir = (end.location() - start.location()).normalized();
        let start_connector_point = start.display().closest_boundary_point(dir);
        let end_connector_point = end.display().closest_boundary_point(-dir);

        let tip_end = end_connector_point;

        let edge_start = start_connector_point;
        let edge_end = end_connector_point - self.tip_size * dir;

        let stroke_edge = Stroke::new(self.width * ctx.meta.zoom, color);
        let stroke_tip = Stroke::new(0., color);
        if self.order == 0 {
            // draw straight edge

            let line = Shape::line_segment(
                [
                    ctx.meta.canvas_to_screen_pos(edge_start),
                    ctx.meta.canvas_to_screen_pos(edge_end),
                ],
                stroke_edge,
            );
            if !ctx.is_directed {
                return vec![line];
            }

            let tip_start_1 = tip_end - self.tip_size * rotate_vector(dir, self.tip_angle);
            let tip_start_2 = tip_end - self.tip_size * rotate_vector(dir, -self.tip_angle);

            // draw tips for directed edges

            let line_tip = Shape::convex_polygon(
                vec![
                    ctx.meta.canvas_to_screen_pos(tip_end),
                    ctx.meta.canvas_to_screen_pos(tip_start_1),
                    ctx.meta.canvas_to_screen_pos(tip_start_2),
                ],
                color,
                stroke_tip,
            );
            return vec![line, line_tip];
        }

        // draw curved edge

        let dir_perpendicular = Vec2::new(-dir.y, dir.x);
        let center_point = (edge_start + edge_end.to_vec2()).to_vec2() / 2.;
        let control_point =
            (center_point + dir_perpendicular * self.curve_size * self.order as f32).to_pos2();

        let tip_dir = (control_point - tip_end).normalized();

        let arrow_tip_dir_1 = rotate_vector(tip_dir, self.tip_angle) * self.tip_size;
        let arrow_tip_dir_2 = rotate_vector(tip_dir, -self.tip_angle) * self.tip_size;

        let tip_start_1 = tip_end + arrow_tip_dir_1;
        let tip_start_2 = tip_end + arrow_tip_dir_2;

        let edge_end_curved = point_between(tip_start_1, tip_start_2);

        let line_curved = QuadraticBezierShape::from_points_stroke(
            [
                ctx.meta.canvas_to_screen_pos(edge_start),
                ctx.meta.canvas_to_screen_pos(control_point),
                ctx.meta.canvas_to_screen_pos(edge_end_curved),
            ],
            false,
            Color32::TRANSPARENT,
            stroke_edge,
        );

        if !ctx.is_directed {
            return vec![line_curved.into()];
        }

        let line_curved_tip = Shape::convex_polygon(
            vec![
                ctx.meta.canvas_to_screen_pos(tip_end),
                ctx.meta.canvas_to_screen_pos(tip_start_1),
                ctx.meta.canvas_to_screen_pos(tip_start_2),
            ],
            color,
            stroke_tip,
        );

        vec![line_curved.into(), line_curved_tip]
    }

    fn update(&mut self, state: &EdgeProps<E>) {
        self.order = state.order;
        self.selected = state.selected;
    }
}

fn shape_looped(
    node_size: f32,
    node_center: Pos2,
    stroke: Stroke,
    e: &DefaultEdgeShape,
) -> CubicBezierShape {
    let center_horizon_angle = PI / 4.;
    let y_intersect = node_center.y - node_size * center_horizon_angle.sin();

    let edge_start = Pos2::new(
        node_center.x - node_size * center_horizon_angle.cos(),
        y_intersect,
    );
    let edge_end = Pos2::new(
        node_center.x + node_size * center_horizon_angle.cos(),
        y_intersect,
    );

    let loop_size = node_size * (e.loop_size + e.order as f32);

    let control_point1 = Pos2::new(node_center.x + loop_size, node_center.y - loop_size);
    let control_point2 = Pos2::new(node_center.x - loop_size, node_center.y - loop_size);

    CubicBezierShape::from_points_stroke(
        [edge_end, control_point1, control_point2, edge_start],
        false,
        Color32::default(),
        stroke,
    )
}

fn shape_curved(
    pos_start: Pos2,
    pos_end: Pos2,
    size_start: f32,
    size_end: f32,
    stroke: Stroke,
    e: &DefaultEdgeShape,
) -> QuadraticBezierShape {
    let vec = pos_end - pos_start;
    let dist: f32 = vec.length();
    let dir = vec / dist;

    let start_node_radius_vec = Vec2::new(size_start, size_start) * dir;
    let end_node_radius_vec = Vec2::new(size_end, size_end) * dir;

    let tip_end = pos_start + vec - end_node_radius_vec;

    let edge_start = pos_start + start_node_radius_vec;
    let edge_end = pos_end + end_node_radius_vec;

    let dir_perpendicular = Vec2::new(-dir.y, dir.x);
    let center_point = (edge_start + tip_end.to_vec2()).to_vec2() / 2.0;
    let control_point =
        (center_point + dir_perpendicular * e.curve_size * e.order as f32).to_pos2();

    QuadraticBezierShape::from_points_stroke(
        [edge_start, control_point, edge_end],
        false,
        stroke.color,
        stroke,
    )
}

fn is_inside_loop<E: Clone, N: Clone, Ix: IndexType, Ty: EdgeType, D: DisplayNode<N, E, Ty, Ix>>(
    node: &Node<N, E, Ty, Ix, D>,
    e: &DefaultEdgeShape,
    pos: Pos2,
) -> bool {
    let node_size = node_size(node);

    let shape = shape_looped(node_size, node.location(), Stroke::default(), e);
    is_point_on_cubic_bezier_curve(pos, shape, e.width)
}

fn is_inside_line(pos_start: Pos2, pos_end: Pos2, pos: Pos2, e: &DefaultEdgeShape) -> bool {
    let distance = distance_segment_to_point(pos_start, pos_end, pos);
    distance <= e.width
}

fn is_inside_curve<
    N: Clone,
    E: Clone,
    Ty: EdgeType,
    Ix: IndexType,
    D: DisplayNode<N, E, Ty, Ix>,
>(
    node_start: &Node<N, E, Ty, Ix, D>,
    node_end: &Node<N, E, Ty, Ix, D>,
    e: &DefaultEdgeShape,
    pos: Pos2,
) -> bool {
    let pos_start = node_start.location();
    let pos_end = node_end.location();

    let size_start = node_size(node_start);
    let size_end = node_size(node_end);

    let shape = shape_curved(
        pos_start,
        pos_end,
        size_start,
        size_end,
        Stroke::default(),
        e,
    );
    is_point_on_quadratic_bezier_curve(pos, shape, e.width)
}

fn node_size<N: Clone, E: Clone, Ty: EdgeType, Ix: IndexType, D: DisplayNode<N, E, Ty, Ix>>(
    node: &Node<N, E, Ty, Ix, D>,
) -> f32 {
    let left_dir = Vec2::new(-1., 0.);
    let connector_left = node.display().closest_boundary_point(left_dir);
    let connector_right = node.display().closest_boundary_point(-left_dir);

    (connector_right.x - connector_left.x) / 2.
}

/// Returns the distance from line segment `a``b` to point `c`.
/// Adapted from https://stackoverflow.com/questions/1073336/circle-line-segment-collision-detection-algorithm
fn distance_segment_to_point(a: Pos2, b: Pos2, point: Pos2) -> f32 {
    let ac = point - a;
    let ab = b - a;

    let d = a + proj(ac, ab);

    let ad = d - a;

    let k = if ab.x.abs() > ab.y.abs() {
        ad.x / ab.x
    } else {
        ad.y / ab.y
    };

    if k <= 0.0 {
        return hypot2(point.to_vec2(), a.to_vec2()).sqrt();
    } else if k >= 1.0 {
        return hypot2(point.to_vec2(), b.to_vec2()).sqrt();
    }

    hypot2(point.to_vec2(), d.to_vec2()).sqrt()
}

/// Calculates the square of the Euclidean distance between vectors `a` and `b`.
fn hypot2(a: Vec2, b: Vec2) -> f32 {
    (a - b).dot(a - b)
}

/// Calculates the projection of vector `a` onto vector `b`.
fn proj(a: Vec2, b: Vec2) -> Vec2 {
    let k = a.dot(b) / b.dot(b);
    Vec2::new(k * b.x, k * b.y)
}

fn is_point_on_cubic_bezier_curve(point: Pos2, curve: CubicBezierShape, width: f32) -> bool {
    is_point_on_bezier_curve(point, curve.flatten(Option::new(10.0)), width)
}

fn is_point_on_quadratic_bezier_curve(
    point: Pos2,
    curve: QuadraticBezierShape,
    width: f32,
) -> bool {
    is_point_on_bezier_curve(point, curve.flatten(Option::new(0.3)), width)
}

fn is_point_on_bezier_curve(point: Pos2, curve_points: Vec<Pos2>, width: f32) -> bool {
    let mut previous_point = None;
    for p in curve_points {
        if let Some(pp) = previous_point {
            let distance = distance_segment_to_point(p, pp, point);
            if distance < width {
                return true;
            }
        }
        previous_point = Some(p);
    }
    false
}

/// rotates vector by angle
fn rotate_vector(vec: Vec2, angle: f32) -> Vec2 {
    let cos = angle.cos();
    let sin = angle.sin();
    Vec2::new(cos * vec.x - sin * vec.y, sin * vec.x + cos * vec.y)
}

/// finds point exactly in the middle between 2 points
fn point_between(p1: Pos2, p2: Pos2) -> Pos2 {
    let base = p1 - p2;
    let base_len = base.length();
    let dir = base / base_len;
    p1 - (base_len / 2.) * dir
}

// TODO: check test cases
#[cfg(test)]
mod tests {
    use egui::{Color32, Stroke};

    use super::*;

    #[test]
    fn test_distance_segment_to_point() {
        let segment_1 = Pos2::new(2.0, 2.0);
        let segment_2 = Pos2::new(2.0, 5.0);
        let point = Pos2::new(4.0, 3.0);
        assert_eq!(distance_segment_to_point(segment_1, segment_2, point), 2.0);
    }

    #[test]
    fn test_distance_segment_to_point_on_segment() {
        let segment_1 = Pos2::new(1.0, 2.0);
        let segment_2 = Pos2::new(1.0, 5.0);
        let point = Pos2::new(1.0, 3.0);
        assert_eq!(distance_segment_to_point(segment_1, segment_2, point), 0.0);
    }

    #[test]
    fn test_hypot2() {
        let a = Vec2::new(0.0, 1.0);
        let b = Vec2::new(0.0, 5.0);
        assert_eq!(hypot2(a, b), 16.0);
    }

    #[test]
    fn test_hypot2_no_distance() {
        let a = Vec2::new(0.0, 1.0);
        assert_eq!(hypot2(a, a), 0.0);
    }

    #[test]
    fn test_proj() {
        let a = Vec2::new(5.0, 8.0);
        let b = Vec2::new(10.0, 0.0);
        let result = proj(a, b);
        assert_eq!(result.x, 5.0);
        assert_eq!(result.y, 0.0);
    }

    #[test]
    fn test_proj_orthogonal() {
        let a = Vec2::new(5.0, 0.0);
        let b = Vec2::new(0.0, 5.0);
        let result = proj(a, b);
        assert_eq!(result.x, 0.0);
        assert_eq!(result.y, 0.0);
    }

    #[test]
    fn test_proj_same_vector() {
        let a = Vec2::new(5.3, 4.9);
        assert_eq!(proj(a, a), a);
    }

    #[test]
    fn test_is_point_on_cubic_bezier_curve() {
        let edge_start = Pos2::new(-3.0, 0.0);
        let edge_end = Pos2::new(3.0, 0.0);
        let control_point1 = Pos2::new(-3.0, 3.0);
        let control_point2 = Pos2::new(4.0, 2.0);
        let curve = CubicBezierShape::from_points_stroke(
            [edge_end, control_point1, control_point2, edge_start],
            false,
            Color32::default(),
            Stroke::default(),
        );

        let width = 1.0;
        let p1 = Pos2::new(0.0, 2.0);
        assert!(!is_point_on_cubic_bezier_curve(p1, curve, width));

        let p2 = Pos2::new(2.0, 1.0);
        assert!(!is_point_on_cubic_bezier_curve(p2, curve, width));
    }

    #[test]
    fn test_is_point_on_quadratic_bezier_curve() {
        let edge_start = Pos2::new(0.0, 0.0);
        let edge_end = Pos2::new(20.0, 0.0);
        let control_point = Pos2::new(10.0, 8.0);
        let curve = QuadraticBezierShape::from_points_stroke(
            [edge_start, control_point, edge_end],
            false,
            Color32::default(),
            Stroke::default(),
        );

        let width = 1.0;
        let p1 = Pos2::new(10.0, 4.0);
        assert!(is_point_on_quadratic_bezier_curve(p1, curve, width));

        let p2 = Pos2::new(3.0, 2.0);
        assert!(is_point_on_quadratic_bezier_curve(p2, curve, width));
    }
}
