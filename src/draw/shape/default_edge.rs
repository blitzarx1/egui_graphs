use std::f32::consts::PI;

use egui::{
    epaint::{CubicBezierShape, QuadraticBezierShape},
    Color32, Pos2, Stroke, Vec2,
};
use petgraph::{matrix_graph::Nullable, stable_graph::IndexType, EdgeType};

use crate::{draw::DrawContext, elements::EdgeID, Edge, Graph, Node};

use super::{EdgeDisplay, Interactable};

#[derive(Clone, Debug)]
pub struct DefaultEdgeShape<Ix: IndexType> {
    pub edge_id: EdgeID<Ix>,

    pub width: f32,
    pub tip_size: f32,
    pub tip_angle: f32,
    pub curve_size: f32,
}

impl<E: Clone, Ix: IndexType> From<Edge<E, Ix>> for DefaultEdgeShape<Ix> {
    fn from(value: Edge<E, Ix>) -> Self {
        Self {
            edge_id: value.id(),
            width: value.width(),
            tip_size: value.tip_size(),
            tip_angle: value.tip_angle(),
            curve_size: value.curve_size(),
        }
    }
}

impl<N: Clone, E: Clone, Ty: EdgeType, Ix: IndexType> EdgeDisplay<N, E, Ty, Ix>
    for DefaultEdgeShape<Ix>
{
    fn shapes(
        &self,
        start: Node<N, Ix>,
        end: Node<N, Ix>,
        ctx: &DrawContext<N, E, Ty, Ix>,
    ) -> Vec<egui::Shape> {
        todo!()
    }
}

impl<N: Clone, E: Clone, Ty: EdgeType, Ix: IndexType> Interactable<N, E, Ty, Ix>
    for DefaultEdgeShape<Ix>
{
    fn is_inside(&self, g: &Graph<N, E, Ty, Ix>, pos: egui::Pos2) -> bool {
        let e = g.edge(self.edge_id.idx).unwrap();

        let (idx_start, idx_end) = g.edge_endpoints(self.edge_id.idx).unwrap();
        let node_start = g.node(idx_start).unwrap();
        let node_end = g.node(idx_end).unwrap();

        if idx_start == idx_end {
            return is_inside_loop(node_start, e, pos);
        }

        if self.edge_id.order == 0 {
            return is_inside_line(node_start, node_end, e, pos);
        }

        is_inside_curve(node_start, node_end, e, pos)
    }
}

fn is_inside_line<E: Clone, N: Clone, Ix: IndexType>(
    node_start: &Node<N, Ix>,
    node_end: &Node<N, Ix>,
    e: &Edge<E, Ix>,
    pos: Pos2,
) -> bool {
    let start_pos = node_start.location();
    let end_pos = node_end.location();

    let distance = distance_segment_to_point(start_pos, end_pos, pos);
    distance <= e.width()
}

fn is_inside_loop<E: Clone, N: Clone, Ix: IndexType>(
    node: &Node<N, Ix>,
    e: &Edge<E, Ix>,
    pos: Pos2,
) -> bool {
    let rad = node.radius();
    let center = node.location();
    let center_horizon_angle = PI / 4.;
    let y_intersect = center.y - rad * center_horizon_angle.sin();

    let edge_start = Pos2::new(center.x - rad * center_horizon_angle.cos(), y_intersect);
    let edge_end = Pos2::new(center.x + rad * center_horizon_angle.cos(), y_intersect);

    let loop_size = rad * (e.style().loop_size + e.id().order as f32);

    let control_point1 = Pos2::new(center.x + loop_size, center.y - loop_size);
    let control_point2 = Pos2::new(center.x - loop_size, center.y - loop_size);

    let shape = CubicBezierShape::from_points_stroke(
        [edge_end, control_point1, control_point2, edge_start],
        false,
        Color32::default(),
        Stroke::default(),
    );

    is_point_on_cubic_bezier_curve(pos, shape, e.width())
}

fn is_inside_curve<E: Clone, N: Clone, Ix: IndexType>(
    node_start: &Node<N, Ix>,
    node_end: &Node<N, Ix>,
    e: &Edge<E, Ix>,
    pos: Pos2,
) -> bool {
    let pos_start = node_start.location();
    let pos_end = node_end.location();
    let rad_start = node_start.radius();
    let rad_end = node_end.radius();

    let vec = pos_end - pos_start;
    let dist: f32 = vec.length();
    let dir = vec / dist;

    let start_node_radius_vec = Vec2::new(rad_start, rad_start) * dir;
    let end_node_radius_vec = Vec2::new(rad_end, rad_end) * dir;

    let tip_end = pos_start + vec - end_node_radius_vec;

    let edge_start = pos_start + start_node_radius_vec;

    let dir_perpendicular = Vec2::new(-dir.y, dir.x);
    let center_point = (edge_start + tip_end.to_vec2()).to_vec2() / 2.0;
    let control_point =
        (center_point + dir_perpendicular * e.curve_size() * e.id().order as f32).to_pos2();

    let tip_vec = control_point - tip_end;
    let tip_dir = tip_vec / tip_vec.length();
    let tip_size = e.tip_size();

    let arrow_tip_dir_1 = rotate_vector(tip_dir, e.tip_angle()) * tip_size;
    let arrow_tip_dir_2 = rotate_vector(tip_dir, -e.tip_angle()) * tip_size;

    let tip_start_1 = tip_end + arrow_tip_dir_1;
    let tip_start_2 = tip_end + arrow_tip_dir_2;

    let edge_end_curved = point_between(tip_start_1, tip_start_2);

    let shape = QuadraticBezierShape::from_points_stroke(
        [edge_start, control_point, edge_end_curved],
        false,
        Color32::default(),
        Stroke::default(),
    );

    is_point_on_quadratic_bezier_curve(pos, shape, e.width())
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
