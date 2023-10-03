use std::{collections::HashMap, f32::consts::PI};

use egui::{
    epaint::{CircleShape, CubicBezierShape, QuadraticBezierShape, TextShape},
    Color32, Context, FontFamily, FontId, Painter, Pos2, Shape, Stroke, Vec2,
};
use petgraph::{stable_graph::NodeIndex, EdgeType};

use crate::{settings::SettingsStyle, Edge, Graph, Metadata, Node};

use super::layers::Layers;

/// Mapping for 2 nodes and all edges between them
type EdgeMap<'a, E> = HashMap<(NodeIndex, NodeIndex), Vec<Edge<E>>>;

/// Custom node draw function
pub type FnCustomNodeDraw<N> = fn(&Context, n: &Node<N>, &Metadata, &SettingsStyle, &mut Layers);

pub struct Drawer<'a, N: Clone, E: Clone, Ty: EdgeType> {
    p: Painter,

    g: &'a Graph<N, E, Ty>,
    style: &'a SettingsStyle,
    meta: &'a Metadata,

    custom_node_draw: Option<FnCustomNodeDraw<N>>,
}

impl<'a, N: Clone, E: Clone, Ty: EdgeType> Drawer<'a, N, E, Ty> {
    pub fn new(
        p: Painter,
        g: &'a Graph<N, E, Ty>,
        style: &'a SettingsStyle,
        meta: &'a Metadata,
        custom_node_draw: Option<FnCustomNodeDraw<N>>,
    ) -> Self {
        Drawer {
            g,
            p,
            style,
            meta,
            custom_node_draw,
        }
    }

    pub fn draw(self) {
        let mut l = Layers::default();

        self.fill_layers_edges(&mut l);
        self.fill_layers_nodes(&mut l);

        l.draw(self.p)
    }

    fn fill_layers_nodes(&self, l: &mut Layers) {
        self.g
            .nodes_iter()
            .for_each(|(_, n)| match self.custom_node_draw {
                Some(f) => f(self.p.ctx(), n, self.meta, self.style, l),
                None => self.default_node_draw(self.p.ctx(), n, self.meta, self.style, l),
            });
    }

    fn fill_layers_edges(&self, l: &mut Layers) {
        let mut edge_map: EdgeMap<E> = HashMap::new();

        self.g.edges_iter().for_each(|(idx, e)| {
            let (source, target) = self.g.edge_endpoints(idx).unwrap();
            // compute map with edges between 2 nodes
            edge_map
                .entry((source, target))
                .or_insert_with(Vec::new)
                .push(e.clone());
        });

        edge_map.iter().for_each(|((start, end), edges)| {
            let mut order = edges.len();
            edges.iter().for_each(|e| {
                order -= 1;

                if start == end {
                    self.draw_edge_looped(l, start, e, order);
                } else {
                    self.draw_edge_basic(l, start, end, e, order);
                }
            });
        });
    }

    fn draw_edge_looped(&self, l: &mut Layers, n_idx: &NodeIndex, e: &Edge<E>, order: usize) {
        let node = self.g.node(*n_idx).unwrap();

        let rad = node.screen_radius(self.meta);
        let center = node.screen_location(self.meta);
        let center_horizon_angle = PI / 4.;
        let y_intersect = center.y - rad * center_horizon_angle.sin();

        let edge_start = Pos2::new(center.x - rad * center_horizon_angle.cos(), y_intersect);
        let edge_end = Pos2::new(center.x + rad * center_horizon_angle.cos(), y_intersect);

        let loop_size = rad * (self.style.edge_looped_size + order as f32);

        let control_point1 = Pos2::new(center.x + loop_size, center.y - loop_size);
        let control_point2 = Pos2::new(center.x - loop_size, center.y - loop_size);

        let stroke = Stroke::new(e.width() * self.meta.zoom, e.color(self.p.ctx()));
        let shape = CubicBezierShape::from_points_stroke(
            [edge_end, control_point1, control_point2, edge_start],
            false,
            Color32::TRANSPARENT,
            stroke,
        );

        l.add(shape);
    }

    fn draw_edge_basic(
        &self,
        l: &mut Layers,
        start_idx: &NodeIndex,
        end_idx: &NodeIndex,
        e: &Edge<E>,
        order: usize,
    ) {
        let n_start = self.g.node(*start_idx).unwrap();
        let n_end = self.g.node(*end_idx).unwrap();

        let loc_start = n_start.screen_location(self.meta).to_pos2();
        let loc_end = n_end.screen_location(self.meta).to_pos2();
        let rad_start = n_start.screen_radius(self.meta);
        let rad_end = n_end.screen_radius(self.meta);

        let vec = loc_end - loc_start;
        let dist: f32 = vec.length();
        let dir = vec / dist;

        let start_node_radius_vec = Vec2::new(rad_start, rad_start) * dir;
        let end_node_radius_vec = Vec2::new(rad_end, rad_end) * dir;

        let tip_end = loc_start + vec - end_node_radius_vec;

        let edge_start = loc_start + start_node_radius_vec;
        let edge_end = match self.g.is_directed() {
            true => tip_end - e.tip_size() * self.meta.zoom * dir,
            false => tip_end,
        };

        let color = e.color(self.p.ctx());
        let stroke_edge = Stroke::new(e.width() * self.meta.zoom, color);
        let stroke_tip = Stroke::new(0., color);

        // draw straight edge
        if order == 0 {
            let tip_start_1 =
                tip_end - e.tip_size() * self.meta.zoom * rotate_vector(dir, e.tip_angle());
            let tip_start_2 =
                tip_end - e.tip_size() * self.meta.zoom * rotate_vector(dir, -e.tip_angle());

            let shape = Shape::line_segment([edge_start, edge_end], stroke_edge);
            l.add(shape);

            // draw tips for directed edges
            if self.g.is_directed() {
                let shape_tip = Shape::convex_polygon(
                    vec![tip_end, tip_start_1, tip_start_2],
                    color,
                    stroke_tip,
                );
                l.add(shape_tip);
            }

            return;
        }

        // draw curved edge
        let dir_perpendicular = Vec2::new(-dir.y, dir.x);
        let center_point = (edge_start + edge_end.to_vec2()).to_vec2() / 2.0;
        let control_point = (center_point
            + dir_perpendicular * e.curve_size() * self.meta.zoom * order as f32)
            .to_pos2();

        let tip_vec = control_point - tip_end;
        let tip_dir = tip_vec / tip_vec.length();
        let tip_size = e.tip_size() * self.meta.zoom;

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

    fn default_node_draw(
        &self,
        ctx: &Context,
        n: &Node<N>,
        m: &Metadata,
        style: &SettingsStyle,
        l: &mut Layers,
    ) {
        let is_interacted = n.selected() || n.dragged();
        let loc = n.screen_location(m).to_pos2();
        let rad = match is_interacted {
            true => n.screen_radius(m) * 1.5,
            false => n.screen_radius(m),
        };

        let color = n.color(ctx);
        let shape_node = CircleShape {
            center: loc,
            radius: rad,
            fill: color,
            stroke: Stroke::new(1., color),
        };
        match is_interacted {
            true => l.add_top(shape_node),
            false => l.add(shape_node),
        };

        let show_label = style.labels_always || is_interacted;
        if !show_label {
            return;
        };

        let color = ctx.style().visuals.text_color();
        let label_pos = Pos2::new(loc.x, loc.y - rad * 2.);
        let label_size = rad;
        let galley = ctx.fonts(|f| {
            f.layout_no_wrap(
                n.label(),
                FontId::new(label_size, FontFamily::Monospace),
                color,
            )
        });

        let shape_label = TextShape::new(label_pos, galley);
        match is_interacted {
            true => l.add_top(shape_label),
            false => l.add(shape_label),
        };
    }
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
