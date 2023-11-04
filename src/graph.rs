use std::f32::consts::PI;
use std::ops::{Index, Sub};
use egui::{Color32, Pos2, Stroke, Vec2};
use egui::epaint::CubicBezierShape;

use petgraph::{
    stable_graph::{EdgeIndex, EdgeReference, NodeIndex, StableGraph},
    visit::{EdgeRef, IntoEdgeReferences, IntoNodeReferences},
    Direction, EdgeType,
};
use petgraph::matrix_graph::Nullable;

use crate::{metadata::Metadata, transform, Edge, Node, SettingsStyle};

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

impl<'a, N: Clone, E: Clone, Ty: EdgeType> Graph<N, E, Ty> {
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

    pub fn edge_by_screen_pos(
        &self,
        meta: &'a Metadata,
        style: &'a SettingsStyle,
        screen_pos: Pos2,
    ) -> Option<(EdgeIndex, &Edge<E>)> {
        let pos_in_graph = (screen_pos.to_vec2() - meta.pan) / meta.zoom;
        self.edges_iter().find(|(idx, e)| {
            let (source, target) = self.g.edge_endpoints(*idx).unwrap();
            let source_pos = self.g.index(source).location();
            let target_pos = self.g.index(target).location();
            if source == target { // edge is a loop (bezier curve)
                // TODO lots of duplicated code (only slightly changed from draw/edge
                let node = self.g.index(source);
                let rad = node.screen_radius(meta, style) / meta.zoom;
                let center = source_pos;
                let center_horizon_angle = PI / 4.;
                let y_intersect = center.y - rad * center_horizon_angle.sin();

                let edge_start = Pos2::new(center.x - rad * center_horizon_angle.cos(), y_intersect);
                let edge_end = Pos2::new(center.x + rad * center_horizon_angle.cos(), y_intersect);

                // TODO what does `order` do?
                let loop_size = rad * style.edge_looped_size;

                let control_point1 = Pos2::new(center.x + loop_size, center.y - loop_size);
                let control_point2 = Pos2::new(center.x - loop_size, center.y - loop_size);

                let stroke = Stroke::new(e.width() * meta.zoom, e.color(&Default::default()));
                let shape = CubicBezierShape::from_points_stroke(
                    [edge_end, control_point1, control_point2, edge_start],
                    false,
                    Color32::TRANSPARENT,
                    stroke,
                );
                let tolerance = Option::new(1.0);
                let mut previous_point = None;
                for p in shape.flatten(tolerance) {
                    if previous_point.is_some() {
                        let distance = distanceSegmentToPoint(p.to_vec2(), previous_point.unwrap(), pos_in_graph);
                        if distance < e.width() {
                            return true;
                        }
                    }
                    previous_point = Option::new(p.to_vec2());
                };
                return false;
            } else { // edge is a straight line between nodes
                let distance = distanceSegmentToPoint(source_pos, target_pos, pos_in_graph);
                distance < e.width()
            }
        })
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

// https://stackoverflow.com/questions/1073336/circle-line-segment-collision-detection-algorithm
/**
 * Returns the distance from line segment ab to point c.
 */
fn distanceSegmentToPoint(a: Vec2, b: Vec2, point: Vec2) -> f32 {
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

    return hypot2(point, d).sqrt();
}

fn hypot2(a: Vec2, b: Vec2) -> f32 {
    return (a - b).dot(a - b);
}

fn proj(a: Vec2, b: Vec2) -> Vec2 {
    let k = a.dot(b) / b.dot(b);
    return Vec2::new(k * b.x, k * b.y);
}
