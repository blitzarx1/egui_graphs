use std::{collections::HashMap, f32::consts::PI};

use egui::{
    epaint::{CircleShape, CubicBezierShape, QuadraticBezierShape, TextShape},
    Color32, FontFamily, FontId, Painter, Pos2, Shape, Stroke, Vec2,
};
use petgraph::{
    stable_graph::{EdgeIndex, NodeIndex},
    EdgeType,
};

use crate::{
    graph_wrapper::GraphWrapper,
    settings::SettingsStyle,
    state_computed::{StateComputed, StateComputedEdge, StateComputedNode},
    Edge, Node,
};

use super::layers::Layers;

/// Edge, its index and computed state
type EdgeWithMeta<'a, E> = (EdgeIndex, Edge<E>, &'a StateComputedEdge);
/// Mapping for 2 nodes and all edges between them
type EdgeMap<'a, E> = HashMap<(NodeIndex, NodeIndex), Vec<EdgeWithMeta<'a, E>>>;

// TODO: support custom drawing functions; check example/basic_custom_drawing for contract
pub struct Drawer<'a, N: Clone, E: Clone, Ty: EdgeType> {
    p: Painter,

    g: &'a GraphWrapper<'a, N, E, Ty>,
    comp: &'a StateComputed,
    settings_style: &'a SettingsStyle,
}

impl<'a, N: Clone, E: Clone, Ty: EdgeType> Drawer<'a, N, E, Ty> {
    pub fn new(
        p: Painter,
        g: &'a GraphWrapper<N, E, Ty>,
        comp: &'a StateComputed,
        settings_style: &'a SettingsStyle,
    ) -> Self {
        Drawer {
            g,
            p,
            comp,
            settings_style,
        }
    }

    pub fn draw(self) {
        let mut l = Layers::default();

        self.fill_layers_edges(&mut l);
        self.fill_layers_nodes(&mut l);

        l.draw(self.p)
    }

    fn fill_layers_nodes(&self, l: &mut Layers) {
        self.g.nodes().for_each(|(idx, n)| {
            let comp_node = self.comp.node_state(&idx).unwrap();
            let loc = comp_node.location.to_pos2();

            self.draw_node_basic(l, loc, n, comp_node);
            self.draw_node_interacted(l, loc, n, comp_node);
        });
    }

    fn fill_layers_edges(&self, l: &mut Layers) {
        let mut edge_map: EdgeMap<E> = HashMap::new();

        self.g.edges().for_each(|(idx, e)| {
            let (source, target) = self.g.edge_endpoints(idx).unwrap();
            // compute map with edges between 2 nodes
            edge_map
                .entry((source, target))
                .or_insert_with(Vec::new)
                .push((idx, e.clone(), self.comp.edge_state(&idx).unwrap()));
        });

        edge_map.iter().for_each(|((start, end), edges)| {
            let mut order = edges.len();
            edges.iter().for_each(|(_, e, comp)| {
                order -= 1;

                if start == end {
                    self.draw_edge_looped(l, start, e, comp, order);
                } else {
                    self.draw_edge_basic(l, start, end, e, comp, order);
                }
            });
        });
    }

    fn draw_edge_looped(
        &self,
        l: &mut Layers,
        n_idx: &NodeIndex,
        e: &Edge<E>,
        comp_edge: &StateComputedEdge,
        order: usize,
    ) {
        let comp_node = self.comp.node_state(n_idx).unwrap();

        if comp_node.subfolded() {
            // we do not draw edges which are folded
            return;
        }

        let center_horizont_angle = PI / 4.;
        let center = comp_node.location;
        let y_intersect = center.y - comp_node.radius * center_horizont_angle.sin();

        let edge_start = Pos2::new(
            center.x - comp_node.radius * center_horizont_angle.cos(),
            y_intersect,
        );
        let edge_end = Pos2::new(
            center.x + comp_node.radius * center_horizont_angle.cos(),
            y_intersect,
        );

        let loop_size = comp_node.radius * (self.settings_style.edge_looped_size + order as f32);

        let control_point1 = Pos2::new(center.x + loop_size, center.y - loop_size);
        let control_point2 = Pos2::new(center.x - loop_size, center.y - loop_size);

        let stroke = Stroke::new(
            comp_edge.width,
            self.settings_style.color_edge(self.p.ctx(), e),
        );
        let shape = CubicBezierShape::from_points_stroke(
            [edge_end, control_point1, control_point2, edge_start],
            false,
            Color32::TRANSPARENT,
            stroke,
        );

        if !comp_edge.subselected() {
            // draw not selected
            l.add_bottom(shape);
            return;
        }

        // draw selected
        let stroke_highlighted = Stroke::new(
            comp_edge.width,
            self.settings_style.color_edge_highlight(comp_edge).unwrap(),
        );
        let shape_selected = CubicBezierShape::from_points_stroke(
            [edge_end, control_point1, control_point2, edge_start],
            false,
            Color32::TRANSPARENT,
            stroke_highlighted,
        );
        l.add_top(shape_selected);
    }

    fn draw_edge_basic(
        &self,
        l: &mut Layers,
        start_idx: &NodeIndex,
        end_idx: &NodeIndex,
        e: &Edge<E>,
        comp_edge: &StateComputedEdge,
        order: usize,
    ) {
        let mut comp_start = self.comp.node_state(start_idx).unwrap();
        let mut comp_end = self.comp.node_state(end_idx).unwrap();
        let start_node = self.g.node(*start_idx).unwrap();
        let mut transparent = false;

        if (start_node.folded() || comp_start.subfolded()) && comp_end.subfolded() {
            return;
        }

        if comp_start.subfolded() && !comp_end.subfolded() {
            let new_start_idx = self
                .comp
                .foldings
                .roots_by_node(*start_idx)
                .unwrap()
                .first()
                .unwrap();
            comp_start = self.comp.node_state(new_start_idx).unwrap();
            transparent = true;
        }

        if !comp_start.subfolded() && comp_end.subfolded() {
            let new_end_idx = self
                .comp
                .foldings
                .roots_by_node(*end_idx)
                .unwrap()
                .first()
                .unwrap();
            comp_end = self.comp.node_state(new_end_idx).unwrap();
            transparent = true;
        }

        let pos_start = comp_start.location.to_pos2();
        let pos_end = comp_end.location.to_pos2();

        let vec = pos_end - pos_start;
        let dist: f32 = vec.length();
        let dir = vec / dist;

        let start_node_radius_vec = Vec2::new(comp_start.radius, comp_start.radius) * dir;
        let end_node_radius_vec = Vec2::new(comp_end.radius, comp_end.radius) * dir;

        let tip_end = pos_start + vec - end_node_radius_vec;

        let edge_start = pos_start + start_node_radius_vec;
        let edge_end = match self.g.is_directed() {
            true => tip_end - comp_edge.tip_size * dir,
            false => tip_end,
        };

        let mut color = self.settings_style.color_edge(self.p.ctx(), e);
        if transparent {
            color = color.gamma_multiply(0.15);
        }

        let stroke_edge = Stroke::new(comp_edge.width, color);
        let stroke_tip = Stroke::new(0., color);

        // draw straight edge
        if order == 0 {
            let tip_start_1 = tip_end - comp_edge.tip_size * rotate_vector(dir, e.tip_angle());
            let tip_start_2 = tip_end - comp_edge.tip_size * rotate_vector(dir, -e.tip_angle());

            if !comp_edge.subselected() {
                //draw straight not selected
                let shape = Shape::line_segment([edge_start, edge_end], stroke_edge);
                l.add_bottom(shape);

                // draw tips for directed edges
                if self.g.is_directed() {
                    let shape_tip = Shape::convex_polygon(
                        vec![tip_end, tip_start_1, tip_start_2],
                        color,
                        stroke_tip,
                    );
                    l.add_bottom(shape_tip);
                }

                return;
            }

            // draw straight selected
            let color_higlight = self.settings_style.color_edge_highlight(comp_edge).unwrap();
            let stroke_edge_highlighted = Stroke::new(comp_edge.width, color_higlight);
            let stroke_tip_highlighted = Stroke::new(0., color_higlight);
            let shape_selected =
                Shape::line_segment([edge_start, edge_end], stroke_edge_highlighted);

            l.add_top(shape_selected);

            if self.g.is_directed() {
                let shape_tip = Shape::convex_polygon(
                    vec![tip_end, tip_start_1, tip_start_2],
                    color_higlight,
                    stroke_tip_highlighted,
                );
                l.add_top(shape_tip)
            }

            return;
        }

        // draw curved edge
        let dir_perpendicular = Vec2::new(-dir.y, dir.x);
        let center_point = (edge_start + edge_end.to_vec2()).to_vec2() / 2.0;
        let control_point =
            (center_point + dir_perpendicular * comp_edge.curve_size * order as f32).to_pos2();

        let tip_vec = control_point - tip_end;
        let tip_dir = tip_vec / tip_vec.length();
        let tip_size = comp_edge.tip_size;

        let arrow_tip_dir_1 = rotate_vector(tip_dir, e.tip_angle()) * tip_size;
        let arrow_tip_dir_2 = rotate_vector(tip_dir, -e.tip_angle()) * tip_size;

        let tip_start_1 = tip_end + arrow_tip_dir_1;
        let tip_start_2 = tip_end + arrow_tip_dir_2;

        let edge_end_curved = point_between(tip_start_1, tip_start_2);

        if !comp_edge.subselected() {
            // draw curved not selected
            let shape_curved = QuadraticBezierShape::from_points_stroke(
                [edge_start, control_point, edge_end_curved],
                false,
                Color32::TRANSPARENT,
                stroke_edge,
            );
            l.add_bottom(shape_curved);

            let shape_tip_curved =
                Shape::convex_polygon(vec![tip_end, tip_start_1, tip_start_2], color, stroke_tip);
            l.add_bottom(shape_tip_curved);

            return;
        }

        // draw curved selected
        let mut color_highlighted = self.settings_style.color_edge_highlight(comp_edge).unwrap();
        if transparent {
            color_highlighted = color_highlighted.gamma_multiply(0.15);
        }
        let stroke_highlighted_edge = Stroke::new(comp_edge.width, color_highlighted);
        let stroke_highlighted_tip = Stroke::new(0., color_highlighted);
        let shape_curved_selected = QuadraticBezierShape::from_points_stroke(
            [edge_start, control_point, edge_end_curved],
            false,
            Color32::TRANSPARENT,
            stroke_highlighted_edge,
        );
        l.add_top(shape_curved_selected);

        let shape_curved_tip_selected = Shape::convex_polygon(
            vec![tip_end, tip_start_1, tip_start_2],
            color_highlighted,
            stroke_highlighted_tip,
        );
        l.add_top(shape_curved_tip_selected);
    }

    fn shape_label(&self, node_radius: f32, loc: Pos2, n: &Node<N>) -> Option<TextShape> {
        let color_label = self.settings_style.color_label(self.p.ctx());
        let label_pos = Pos2::new(loc.x, loc.y - node_radius * 2.);
        let label_size = node_radius;
        let galley = self.p.layout_no_wrap(
            n.label()?.clone(),
            FontId::new(label_size, FontFamily::Monospace),
            color_label,
        );

        Some(TextShape::new(label_pos, galley))
    }

    fn draw_node_basic(
        &self,
        l: &mut Layers,
        loc: Pos2,
        node: &Node<N>,
        comp_node: &StateComputedNode,
    ) {
        let color = self.settings_style.color_node(self.p.ctx(), node);
        let node_radius = comp_node.radius;
        let shape = CircleShape {
            center: loc,
            radius: node_radius,
            fill: color,
            stroke: Stroke::new(1., color),
        };

        if !(node.selected()
            || comp_node.subselected()
            || node.dragged()
            || node.folded()
            || comp_node.subfolded())
        {
            l.add_bottom(shape);

            if !self.settings_style.labels_always {
                return;
            }

            if let Some(shape_label) = self.shape_label(node_radius, loc, node) {
                l.add_bottom(shape_label);
            }

            return;
        }

        if node.folded() || comp_node.subfolded() {
            return;
        }

        l.add_top(shape)
    }

    fn draw_node_interacted(
        &self,
        l: &mut Layers,
        loc: Pos2,
        node: &Node<N>,
        comp_node: &StateComputedNode,
    ) {
        if !(node.selected()
            || comp_node.subselected()
            || node.dragged()
            || node.folded()
            || comp_node.subfolded())
        {
            return;
        }

        if comp_node.subfolded() {
            return;
        }

        let node_radius = comp_node.radius;
        let highlight_radius = node_radius * 1.5;
        let text_size = node_radius / 2.;

        if let Some(shape_label) = self.shape_label(node_radius, loc, node) {
            l.add_top(shape_label);
        };

        // TODO: think of folded and selected visualisation

        if node.folded() {
            let shape_folded = CircleShape::stroke(
                loc,
                node_radius,
                Stroke::new(1., self.settings_style.color_node(self.p.ctx(), node)),
            );
            let galley = self.p.layout_no_wrap(
                comp_node.num_folded.to_string(),
                FontId::monospace(text_size),
                self.settings_style.color_label(self.p.ctx()),
            );
            let galley_offset = node_radius / 4.;
            let galley_pos = Pos2::new(loc.x - galley_offset, loc.y - galley_offset);
            let shape_galley = TextShape::new(galley_pos, galley);
            l.add_top(shape_folded);
            l.add_top(shape_galley);
            return;
        }

        let shape = CircleShape {
            center: loc,
            radius: highlight_radius,
            fill: Color32::TRANSPARENT,
            stroke: Stroke::new(
                node_radius,
                self.settings_style
                    .color_node_highlight(node, comp_node)
                    .unwrap(),
            ),
        };

        l.add_top(shape);
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
        let angle = std::f32::consts::PI / 2.0;
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
