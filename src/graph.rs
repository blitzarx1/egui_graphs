use egui::epaint::{CubicBezierShape, QuadraticBezierShape};
use egui::{Color32, Pos2, Stroke, Vec2};
use petgraph::stable_graph::DefaultIx;
use petgraph::Directed;
use std::f32::consts::PI;
use std::ops::Index;

use petgraph::graph::IndexType;
use petgraph::matrix_graph::Nullable;
use petgraph::{
    stable_graph::{EdgeIndex, EdgeReference, NodeIndex, StableGraph},
    visit::{EdgeRef, IntoEdgeReferences, IntoNodeReferences},
    Direction, EdgeType,
};

use crate::draw::NodeDisplay;
use crate::{metadata::Metadata, transform, Edge, Node};

/// Graph type compatible with [`super::GraphView`].
#[derive(Debug, Clone)]
pub struct Graph<N: Clone, E: Clone, Ty: EdgeType = Directed, Ix: IndexType = DefaultIx> {
    pub g: StableGraph<Node<N, Ix>, Edge<E, Ix>, Ty, Ix>,
}

impl<N: Clone, E: Clone, Ty: EdgeType, Ix: IndexType> From<&StableGraph<N, E, Ty, Ix>>
    for Graph<N, E, Ty, Ix>
{
    fn from(value: &StableGraph<N, E, Ty, Ix>) -> Self {
        transform::to_graph(value)
    }
}

impl<'a, N: Clone, E: Clone + 'a, Ty: EdgeType, Ix: IndexType> Graph<N, E, Ty, Ix> {
    pub fn new(g: StableGraph<Node<N, Ix>, Edge<E, Ix>, Ty, Ix>) -> Self {
        Self { g }
    }

    /// Finds node by position. Can be optimized by using a spatial index like quad-tree if needed.
    pub fn node_by_screen_pos<D: NodeDisplay<N, E, Ty, Ix>>(
        &self,
        meta: &'a Metadata,
        screen_pos: Pos2,
    ) -> Option<(NodeIndex<Ix>, &Node<N, Ix>)> {
        let pos_in_graph = ((screen_pos.to_vec2() - meta.pan) / meta.zoom).to_pos2();
        self.nodes_iter()
            .find(|(_, n)| D::from(n.clone().clone()).is_inside(self, pos_in_graph))
    }

    /// Finds edge by position.
    pub fn edge_by_screen_pos(
        &self,
        meta: &'a Metadata,
        screen_pos: Pos2,
    ) -> Option<EdgeIndex<Ix>> {
        let pos_in_graph = meta.screen_to_canvas_pos(screen_pos);
        for (idx_edge, e) in self.edges_iter() {
            let (start, end) = self.edge_endpoints(e.id().idx).unwrap();

            let pos_start = self.g.index(start).location();
            let pos_end = self.g.index(end).location();

            let node_start = self.g.index(start);
            let node_end = self.g.index(end);

            if start == end {
                // edge is a loop (bezier curve)
                let rad = meta.canvas_to_screen_size(node_start.radius()) / meta.zoom;
                let center = pos_start;
                let center_horizon_angle = PI / 4.;
                let y_intersect = center.y - rad * center_horizon_angle.sin();

                let edge_start =
                    Pos2::new(center.x - rad * center_horizon_angle.cos(), y_intersect);
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
                if is_point_on_cubic_bezier_curve(pos_in_graph, shape, e.width(), meta.zoom) {
                    return Option::new(idx_edge);
                }

                continue;
            }

            if e.id().order == 0 {
                // edge is a straight line between nodes
                let distance = distance_segment_to_point(pos_start, pos_end, pos_in_graph);
                if distance < e.width() {
                    return Option::new(idx_edge);
                }

                continue;
            }

            // multiple edges between nodes -> curved
            let rad_start = meta.canvas_to_screen_size(node_start.radius()) / meta.zoom;
            let rad_end = meta.canvas_to_screen_size(node_end.radius()) / meta.zoom;

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
            if is_point_on_quadratic_bezier_curve(pos_in_graph, shape, e.width(), meta.zoom) {
                return Option::new(idx_edge);
            }
        }

        None
    }

    pub fn g(&mut self) -> &mut StableGraph<Node<N, Ix>, Edge<E, Ix>, Ty, Ix> {
        &mut self.g
    }

    ///Provides iterator over all nodes and their indices.
    pub fn nodes_iter(&'a self) -> impl Iterator<Item = (NodeIndex<Ix>, &Node<N, Ix>)> {
        self.g.node_references()
    }

    /// Provides iterator over all edges and their indices.
    pub fn edges_iter(&'a self) -> impl Iterator<Item = (EdgeIndex<Ix>, &Edge<E, Ix>)> {
        self.g.edge_references().map(|e| (e.id(), e.weight()))
    }

    pub fn node(&self, i: NodeIndex<Ix>) -> Option<&Node<N, Ix>> {
        self.g.node_weight(i)
    }

    pub fn edge(&self, i: EdgeIndex<Ix>) -> Option<&Edge<E, Ix>> {
        self.g.edge_weight(i)
    }

    pub fn edge_endpoints(&self, i: EdgeIndex<Ix>) -> Option<(NodeIndex<Ix>, NodeIndex<Ix>)> {
        self.g.edge_endpoints(i)
    }

    pub fn node_mut(&mut self, i: NodeIndex<Ix>) -> Option<&mut Node<N, Ix>> {
        self.g.node_weight_mut(i)
    }

    pub fn edge_mut(&mut self, i: EdgeIndex<Ix>) -> Option<&mut Edge<E, Ix>> {
        self.g.edge_weight_mut(i)
    }

    pub fn is_directed(&self) -> bool {
        self.g.is_directed()
    }

    pub fn edges_num(&self, idx: NodeIndex<Ix>) -> usize {
        self.g.edges(idx).count()
    }

    pub fn edges_directed(
        &self,
        idx: NodeIndex<Ix>,
        dir: Direction,
    ) -> impl Iterator<Item = EdgeReference<Edge<E, Ix>, Ix>> {
        self.g.edges_directed(idx, dir)
    }
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
