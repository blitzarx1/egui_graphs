use egui::epaint::{CubicBezierShape, QuadraticBezierShape};
use egui::{Color32, Pos2, Stroke, Vec2};
use std::collections::HashMap;
use std::f32::consts::PI;
use std::ops::Index;

use petgraph::matrix_graph::Nullable;
use petgraph::{
    stable_graph::{EdgeIndex, EdgeReference, NodeIndex, StableGraph},
    visit::{EdgeRef, IntoEdgeReferences, IntoNodeReferences},
    Direction, EdgeType,
};

use crate::{metadata::Metadata, transform, Edge, Node, SettingsStyle};

/// Mapping for 2 nodes and all edges between them
pub type EdgeMap<'a, E> = HashMap<(NodeIndex, NodeIndex), Vec<(EdgeIndex, &'a Edge<E>)>>;

/// Graph type compatible with [`super::GraphView`].
#[derive(Debug, Clone)]
pub struct Graph<N: Clone, E: Clone, Ty: EdgeType> {
    pub g: StableGraph<Node<N>, Edge<E>, Ty>,
}

impl<N: Clone, E: Clone, Ty: EdgeType> From<&StableGraph<N, E, Ty>> for Graph<N, E, Ty> {
    fn from(value: &StableGraph<N, E, Ty>) -> Self {
        transform::to_graph(value)
    }
}

impl<'a, N: Clone, E: Clone + 'a, Ty: EdgeType> Graph<N, E, Ty> {
    pub fn new(g: StableGraph<Node<N>, Edge<E>, Ty>) -> Self {
        Self { g }
    }

    /// Finds node by position. Can be optimized by using a spatial index like quad-tree if needed.
    pub fn node_by_screen_pos(
        &self,
        meta: &'a Metadata,
        style: &'a SettingsStyle,
        screen_pos: Pos2,
    ) -> Option<(NodeIndex, &Node<N>)> {
        let pos_in_graph = (screen_pos.to_vec2() - meta.pan) / meta.zoom;
        self.nodes_iter().find(|(_, n)| {
            let dist_to_node = (n.location() - pos_in_graph).length();
            dist_to_node <= n.screen_radius(meta, style) / meta.zoom
        })
    }

    /// Finds edge by position.
    pub fn edge_by_screen_pos(
        &self,
        meta: &'a Metadata,
        style: &'a SettingsStyle,
        screen_pos: Pos2,
        edge_map: EdgeMap<E>,
    ) -> Option<EdgeIndex> {
        let pos_in_graph = (screen_pos.to_vec2() - meta.pan) / meta.zoom;
        for ((start, end), edges) in edge_map {
            let mut order = edges.len();
            for (idx_edge, e) in edges {
                let pos_start = self.g.index(start).location().to_pos2();
                let pos_end = self.g.index(end).location().to_pos2();

                let node_start = self.g.index(start);
                let node_end = self.g.index(end);

                order -= 1;

                if start == end {
                    // edge is a loop (bezier curve)
                    let rad = node_start.screen_radius(meta, style) / meta.zoom;
                    let center = pos_start;
                    let center_horizon_angle = PI / 4.;
                    let y_intersect = center.y - rad * center_horizon_angle.sin();

                    let edge_start =
                        Pos2::new(center.x - rad * center_horizon_angle.cos(), y_intersect);
                    let edge_end =
                        Pos2::new(center.x + rad * center_horizon_angle.cos(), y_intersect);

                    let loop_size = rad * (style.edge_looped_size + order as f32);

                    let control_point1 = Pos2::new(center.x + loop_size, center.y - loop_size);
                    let control_point2 = Pos2::new(center.x - loop_size, center.y - loop_size);

                    let stroke = Stroke::new(e.width() * meta.zoom, e.color(&Default::default()));
                    let shape = CubicBezierShape::from_points_stroke(
                        [edge_end, control_point1, control_point2, edge_start],
                        false,
                        Color32::TRANSPARENT,
                        stroke,
                    );
                    if is_point_on_cubic_bezier_curve(pos_in_graph, shape, e.width(), meta.zoom) {
                        return Option::new(idx_edge);
                    }

                    return None;
                }

                if order == 0 {
                    // edge is a straight line between nodes
                    let distance = distance_segment_to_point(
                        pos_start.to_vec2(),
                        pos_end.to_vec2(),
                        pos_in_graph,
                    );
                    if distance < e.width() {
                        return Option::new(idx_edge);
                    }

                    return None;
                }

                // multiple edges between nodes -> curved
                let rad_start = node_start.screen_radius(meta, style) / meta.zoom;
                let rad_end = node_end.screen_radius(meta, style) / meta.zoom;

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
                    (center_point + dir_perpendicular * e.curve_size() * order as f32).to_pos2();

                let stroke = Stroke::new(e.width() * meta.zoom, e.color(&Default::default()));
                let shape = QuadraticBezierShape::from_points_stroke(
                    [edge_start, control_point, tip_end],
                    false,
                    Color32::TRANSPARENT,
                    stroke,
                );
                if is_point_on_quadratic_bezier_curve(pos_in_graph, shape, e.width(), meta.zoom) {
                    return Option::new(idx_edge);
                }
            }
        }

        None
    }

    pub fn g(&mut self) -> &mut StableGraph<Node<N>, Edge<E>, Ty> {
        &mut self.g
    }

    ///Provides iterator over all nodes and their indices.
    pub fn nodes_iter(&'a self) -> impl Iterator<Item = (NodeIndex, &Node<N>)> {
        self.g.node_references()
    }

    /// Provides iterator over all edges and their indices.
    pub fn edges_iter(&'a self) -> impl Iterator<Item = (EdgeIndex, &Edge<E>)> {
        self.g.edge_references().map(|e| (e.id(), e.weight()))
    }

    pub fn node(&self, i: NodeIndex) -> Option<&Node<N>> {
        self.g.node_weight(i)
    }

    pub fn edge(&self, i: EdgeIndex) -> Option<&Edge<E>> {
        self.g.edge_weight(i)
    }

    pub fn edge_endpoints(&self, i: EdgeIndex) -> Option<(NodeIndex, NodeIndex)> {
        self.g.edge_endpoints(i)
    }

    pub fn node_mut(&mut self, i: NodeIndex) -> Option<&mut Node<N>> {
        self.g.node_weight_mut(i)
    }

    pub fn edge_mut(&mut self, i: EdgeIndex) -> Option<&mut Edge<E>> {
        self.g.edge_weight_mut(i)
    }

    pub fn is_directed(&self) -> bool {
        self.g.is_directed()
    }

    pub fn edges_num(&self, idx: NodeIndex) -> usize {
        self.g.edges(idx).count()
    }

    pub fn edges_directed(
        &self,
        idx: NodeIndex,
        dir: Direction,
    ) -> impl Iterator<Item = EdgeReference<Edge<E>>> {
        self.g.edges_directed(idx, dir)
    }
}

/// Returns the distance from line segment ab to point c.
/// Adapted from https://stackoverflow.com/questions/1073336/circle-line-segment-collision-detection-algorithm
fn distance_segment_to_point(a: Vec2, b: Vec2, point: Vec2) -> f32 {
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
        return hypot2(point, a).sqrt();
    } else if k >= 1.0 {
        return hypot2(point, b).sqrt();
    }

    hypot2(point, d).sqrt()
}

fn hypot2(a: Vec2, b: Vec2) -> f32 {
    (a - b).dot(a - b)
}

fn proj(a: Vec2, b: Vec2) -> Vec2 {
    let k = a.dot(b) / b.dot(b);
    Vec2::new(k * b.x, k * b.y)
}

fn is_point_on_cubic_bezier_curve(
    point: Vec2,
    curve: CubicBezierShape,
    width: f32,
    zoom: f32,
) -> bool {
    is_point_on_bezier_curve(point, curve.flatten(Option::new(10.0 / zoom)), width)
}

fn is_point_on_quadratic_bezier_curve(
    point: Vec2,
    curve: QuadraticBezierShape,
    width: f32,
    zoom: f32,
) -> bool {
    is_point_on_bezier_curve(point, curve.flatten(Option::new(2.0 / zoom)), width)
}

fn is_point_on_bezier_curve(point: Vec2, curve_points: Vec<Pos2>, width: f32) -> bool {
    let mut previous_point = None;
    for p in curve_points {
        if previous_point.is_none() {
            previous_point = Some(p.to_vec2());
            continue;
        }

        let distance = distance_segment_to_point(p.to_vec2(), previous_point.unwrap(), point);
        if distance < width {
            return true;
        }
    }
    false
}
