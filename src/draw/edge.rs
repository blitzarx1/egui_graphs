use std::f32::consts::PI;

use egui::{
    epaint::{CubicBezierShape, QuadraticBezierShape},
    Color32, Context, Pos2, Shape, Stroke, Vec2,
};
use petgraph::{stable_graph::NodeIndex, EdgeType};

use crate::{Edge, Node};

use super::{custom::WidgetState, Layers};

pub fn default_edges_draw<N: Clone, E: Clone, Ty: EdgeType>(
    ctx: &Context,
    bounds: (NodeIndex, NodeIndex),
    edges: Vec<&Edge<E>>,
    state: &WidgetState<N, E, Ty>,
    l: &mut Layers,
) {
    let (idx_start, idx_end) = bounds;
    let mut order = edges.len();
    edges.iter().for_each(|e| {
        let n_start = state.g.node(idx_start).unwrap();
        let n_end = state.g.node(idx_end).unwrap();

        order -= 1;

        if idx_start == idx_end {
            draw_edge_looped(ctx, l, n_start, e, order, state);
        } else {
            draw_edge_basic(ctx, l, n_start, n_end, e, order, state);
        }
    });
}

fn draw_edge_basic<N: Clone, E: Clone, Ty: EdgeType>(
    ctx: &Context,
    l: &mut Layers,
    n_start: &Node<N>,
    n_end: &Node<N>,
    e: &Edge<E>,
    order: usize,
    state: &WidgetState<N, E, Ty>,
) {
    let loc_start = n_start.screen_location(state.meta).to_pos2();
    let loc_end = n_end.screen_location(state.meta).to_pos2();
    let rad_start = n_start.screen_radius(state.meta, state.style);
    let rad_end = n_end.screen_radius(state.meta, state.style);

    let vec = loc_end - loc_start;
    let dist: f32 = vec.length();
    let dir = vec / dist;

    let start_node_radius_vec = Vec2::new(rad_start, rad_start) * dir;
    let end_node_radius_vec = Vec2::new(rad_end, rad_end) * dir;

    let tip_end = loc_start + vec - end_node_radius_vec;

    let edge_start = loc_start + start_node_radius_vec;
    let edge_end = match state.g.is_directed() {
        true => tip_end - e.tip_size() * state.meta.zoom * dir,
        false => tip_end,
    };

    let color = e.color(ctx);
    let stroke_edge = Stroke::new(e.width() * state.meta.zoom, color);
    let stroke_tip = Stroke::new(0., color);

    // draw straight edge
    if order == 0 {
        let tip_start_1 =
            tip_end - e.tip_size() * state.meta.zoom * rotate_vector(dir, e.tip_angle());
        let tip_start_2 =
            tip_end - e.tip_size() * state.meta.zoom * rotate_vector(dir, -e.tip_angle());

        let shape = Shape::line_segment([edge_start, edge_end], stroke_edge);
        l.add(shape);

        // draw tips for directed edges
        if state.g.is_directed() {
            let shape_tip =
                Shape::convex_polygon(vec![tip_end, tip_start_1, tip_start_2], color, stroke_tip);
            l.add(shape_tip);
        }

        return;
    }

    // draw curved edge
    let dir_perpendicular = Vec2::new(-dir.y, dir.x);
    let center_point = (edge_start + edge_end.to_vec2()).to_vec2() / 2.0;
    let control_point = (center_point
        + dir_perpendicular * e.curve_size() * state.meta.zoom * order as f32)
        .to_pos2();

    let tip_vec = control_point - tip_end;
    let tip_dir = tip_vec / tip_vec.length();
    let tip_size = e.tip_size() * state.meta.zoom;

    let arrow_tip_dir_1 = rotate_vector(tip_dir, e.tip_angle()) * tip_size;
    let arrow_tip_dir_2 = rotate_vector(tip_dir, -e.tip_angle()) * tip_size;

    let tip_start_1 = tip_end + arrow_tip_dir_1;
    let tip_start_2 = tip_end + arrow_tip_dir_2;

    let edge_end_curved = point_between(tip_start_1, tip_start_2);

    // draw curved not selected
    let shape_curved = QuadraticBezierShape::from_points_stroke(
        [edge_start, control_point, edge_end_curved],
        false,
        Color32::TRANSPARENT,
        stroke_edge,
    );
    l.add(shape_curved);

    let shape_tip_curved =
        Shape::convex_polygon(vec![tip_end, tip_start_1, tip_start_2], color, stroke_tip);
    l.add(shape_tip_curved);
}

fn draw_edge_looped<N: Clone, E: Clone, Ty: EdgeType>(
    ctx: &Context,
    l: &mut Layers,
    node: &Node<N>,
    e: &Edge<E>,
    order: usize,
    state: &WidgetState<N, E, Ty>,
) {
    let rad = node.screen_radius(state.meta, state.style);
    let center = node.screen_location(state.meta);
    let center_horizon_angle = PI / 4.;
    let y_intersect = center.y - rad * center_horizon_angle.sin();

    let edge_start = Pos2::new(center.x - rad * center_horizon_angle.cos(), y_intersect);
    let edge_end = Pos2::new(center.x + rad * center_horizon_angle.cos(), y_intersect);

    let loop_size = rad * (state.style.edge_looped_size + order as f32);

    let control_point1 = Pos2::new(center.x + loop_size, center.y - loop_size);
    let control_point2 = Pos2::new(center.x - loop_size, center.y - loop_size);

    let stroke = Stroke::new(e.width() * state.meta.zoom, e.color(ctx));
    let shape = CubicBezierShape::from_points_stroke(
        [edge_end, control_point1, control_point2, edge_start],
        false,
        Color32::TRANSPARENT,
        stroke,
    );

    l.add(shape);
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
    fn test_rotate_vector() {
        let vec = Vec2::new(1.0, 0.0);
        let angle = PI / 2.0;
        let rotated = rotate_vector(vec, angle);
        assert!((rotated.x - 0.0).abs() < 1e-6);
        assert!((rotated.y - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_point_between() {
        let m = point_between(Pos2::new(0.0, 0.0), Pos2::new(2.0, 0.0));
        assert!((m.x - 1.0).abs() < 1e-6);
        assert!((m.y).abs() < 1e-6);
    }
}
