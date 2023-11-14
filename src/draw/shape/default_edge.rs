use egui::{Pos2, Vec2, epaint::{CubicBezierShape, QuadraticBezierShape}};
use petgraph::{stable_graph::IndexType, EdgeType, matrix_graph::Nullable};

use crate::{elements::EdgeID, Edge, Graph};

use super::Interactable;

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

impl<N: Clone, E: Clone, Ty: EdgeType, Ix: IndexType> Interactable<N, E, Ty, Ix>
    for DefaultEdgeShape<Ix>
{
    fn is_inside(&self, g: &Graph<N, E, Ty, Ix>, pos: egui::Pos2) -> bool {
        let (idx_start, idx_end) = g.edge_endpoints(self.edge_id.idx).unwrap();
        let start = g.node(idx_start).unwrap();
        let end = g.node(idx_end).unwrap();

        if self.edge_id.order == 0 {
            is_inside_line(start.location(), end.location(), pos)
        } else {
            is_inside_curve(start.location(), end.location(), self.edge_id.order, pos)
        }
    }
}

fn is_inside_line(start_loc: Pos2, end_loc: Pos2, pos: Pos2) -> bool {
    !todo!()
}

fn is_inside_curve(start_loc: Pos2, end_loc: Pos2, order: usize, pos: Pos2) -> bool {
    !todo!()
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

fn is_point_on_cubic_bezier_curve(
    point: Pos2,
    curve: CubicBezierShape,
    width: f32,
    zoom: f32,
) -> bool {
    is_point_on_bezier_curve(point, curve.flatten(Option::new(10.0 / zoom)), width)
}

fn is_point_on_quadratic_bezier_curve(
    point: Pos2,
    curve: QuadraticBezierShape,
    width: f32,
    zoom: f32,
) -> bool {
    is_point_on_bezier_curve(point, curve.flatten(Option::new(0.3 / zoom)), width)
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

        let zoom = 5.0;
        let width = 1.0;
        let p1 = Pos2::new(0.0, 2.0);
        assert!(is_point_on_cubic_bezier_curve(p1, curve, width, zoom));

        let p2 = Pos2::new(2.0, 1.0);
        assert!(is_point_on_cubic_bezier_curve(p2, curve, width, zoom));
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

        let zoom = 5.0;
        let width = 1.0;
        let p1 = Pos2::new(10.0, 4.0);
        assert!(is_point_on_quadratic_bezier_curve(p1, curve, width, zoom));

        let p2 = Pos2::new(3.0, 2.0);
        assert!(is_point_on_quadratic_bezier_curve(p2, curve, width, zoom));
    }
}