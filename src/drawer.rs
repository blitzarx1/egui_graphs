use std::{
    collections::HashMap,
    f32::{MAX, MIN},
};

use egui::{
    epaint::{CircleShape, CubicBezierShape, QuadraticBezierShape, TextShape},
    Color32, FontFamily, FontId, Painter, Pos2, Rect, Shape, Stroke, Vec2,
};
use petgraph::{
    stable_graph::{EdgeIndex, NodeIndex},
    visit::EdgeRef,
    EdgeType,
};

use crate::{
    graph_wrapper::GraphWrapper,
    metadata::Metadata,
    state_computed::{StateComputed, StateComputedEdge, StateComputedNode},
    Edge, Node, SettingsStyle,
};

pub struct Drawer<'a, N: Clone, E: Clone, Ty: EdgeType> {
    g: &'a GraphWrapper<'a, N, E, Ty>,
    p: &'a Painter,
    meta: &'a mut Metadata,
    comp: &'a mut StateComputed,
    settings_style: &'a SettingsStyle,
}

impl<'a, N: Clone, E: Clone, Ty: EdgeType> Drawer<'a, N, E, Ty> {
    pub fn new(
        g: &'a GraphWrapper<N, E, Ty>,
        p: &'a Painter,
        meta: &'a mut Metadata,
        comp: &'a mut StateComputed,
        settings_style: &'a SettingsStyle,
    ) -> Self {
        Drawer {
            g,
            p,
            meta,
            comp,
            settings_style,
        }
    }

    pub fn draw(&mut self) {
        let edges_shapes = self.draw_edges();
        let (new_dragged, new_rect, nodes_shapes) = self.draw_nodes();

        self.draw_edges_shapes(edges_shapes);
        self.draw_nodes_shapes(nodes_shapes);

        self.comp.dragged = new_dragged;
        self.meta.graph_bounds = new_rect;
    }

    fn draw_nodes(&self) -> (Option<NodeIndex>, Rect, (Vec<CircleShape>, Vec<TextShape>)) {
        let mut circles = vec![];
        let mut texts = vec![];
        let (mut min_x, mut min_y, mut max_x, mut max_y) = (MAX, MAX, MIN, MIN);
        let mut new_dragged = None;
        self.g
            .nodes_with_context(self.comp)
            .for_each(|(idx, n, comp_node)| {
                // TODO: dont count graph bounds here. Count in computed state instead.

                // update graph bounds on the fly
                // we shall account for the node radius
                // so that the node is fully visible

                let x_minus_rad = n.location.x - comp_node.radius(self.meta);
                if x_minus_rad < min_x {
                    min_x = x_minus_rad;
                };

                let y_minus_rad = n.location.y - comp_node.radius(self.meta);
                if y_minus_rad < min_y {
                    min_y = y_minus_rad;
                };

                let x_plus_rad = n.location.x + comp_node.radius(self.meta);
                if x_plus_rad > max_x {
                    max_x = x_plus_rad;
                };

                let y_plus_rad = n.location.y + comp_node.radius(self.meta);
                if y_plus_rad > max_y {
                    max_y = y_plus_rad;
                };

                if n.dragged {
                    new_dragged = Some(idx);
                }

                let (circles_node, texts_node) = self.draw_node(n, comp_node);
                circles.extend(circles_node);
                texts.extend(texts_node);
            });

        (
            new_dragged,
            Rect::from_min_max(Pos2::new(min_x, min_y), Pos2::new(max_x, max_y)),
            (circles, texts),
        )
    }

    fn draw_edges(&self) -> (Vec<Shape>, Vec<CubicBezierShape>, Vec<QuadraticBezierShape>) {
        let mut shapes = (Vec::new(), Vec::new(), Vec::new());

        let mut edge_map: HashMap<
            (NodeIndex, NodeIndex),
            Vec<(EdgeIndex, Edge<E>, &StateComputedEdge)>,
        > = HashMap::new();

        self.g.edges().for_each(|e| {
            let source = e.source();
            let target = e.target();
            let edge = self.g.edge(e.id()).unwrap();

            edge_map
                .entry((source, target))
                .or_insert_with(Vec::new)
                .push((e.id(), edge.clone(), self.comp.edge_state(&e.id()).unwrap()));
        });

        let edges_by_nodes = edge_map
            .iter()
            .map(|entry| (*entry.0, entry.1.clone()))
            .collect::<Vec<_>>();

        edges_by_nodes.iter().for_each(|((start, end), edges)| {
            let mut order = edges.len();
            edges.iter().for_each(|(_, e, comp)| {
                order -= 1;

                let edge = e.screen_transform(self.meta);

                let selected = self.draw_edge(&edge, comp, start, end, order);
                shapes.0.extend(selected.0);
                shapes.1.extend(selected.1);
                shapes.2.extend(selected.2);
            });
        });

        shapes
    }

    fn draw_edges_shapes(
        &self,
        shapes: (Vec<Shape>, Vec<CubicBezierShape>, Vec<QuadraticBezierShape>),
    ) {
        shapes.0.into_iter().for_each(|shape| {
            self.p.add(shape);
        });
        shapes.1.into_iter().for_each(|shape| {
            self.p.add(shape);
        });
        shapes.2.into_iter().for_each(|shape| {
            self.p.add(shape);
        });
    }

    fn draw_nodes_shapes(&self, shapes: (Vec<CircleShape>, Vec<TextShape>)) {
        shapes.0.into_iter().for_each(|shape| {
            self.p.add(shape);
        });

        shapes.1.into_iter().for_each(|shape| {
            self.p.add(shape);
        });
    }

    fn draw_edge(
        &self,
        edge: &Edge<E>,
        comp_edge: &StateComputedEdge,
        start: &NodeIndex,
        end: &NodeIndex,
        order: usize,
    ) -> (Vec<Shape>, Vec<CubicBezierShape>, Vec<QuadraticBezierShape>) {
        let mut selected_shapes = vec![];
        let mut selected_quadratic = vec![];
        let mut selected_cubic = vec![];

        if start == end {
            self.draw_edge_looped(start, edge, comp_edge, order)
                .into_iter()
                .for_each(|c| {
                    selected_cubic.push(c);
                });

            return (selected_shapes, selected_cubic, selected_quadratic);
        }

        let shapes = self.draw_edge_basic(start, end, edge, comp_edge, order);
        shapes
            .0
            .into_iter()
            .for_each(|shape| selected_shapes.push(shape));
        shapes
            .1
            .into_iter()
            .for_each(|shape| selected_quadratic.push(shape));

        (selected_shapes, selected_cubic, selected_quadratic)
    }

    fn draw_edge_looped(
        &self,
        n_idx: &NodeIndex,
        e: &Edge<E>,
        comp_edge: &StateComputedEdge,
        order: usize,
    ) -> Vec<CubicBezierShape> {
        let n: Node<N> = self.g.node(*n_idx).unwrap().screen_transform(self.meta);
        let comp_node = self.comp.node_state(n_idx).unwrap();

        if comp_node.subfolded() {
            return vec![];
        }

        let pos_start_and_end = n.location.to_pos2();
        let loop_size = comp_node.radius(self.meta) * (4. + 1. + order as f32);

        let control_point1 = Pos2::new(
            pos_start_and_end.x + loop_size,
            pos_start_and_end.y - loop_size,
        );
        let control_point2 = Pos2::new(
            pos_start_and_end.x - loop_size,
            pos_start_and_end.y - loop_size,
        );

        let stroke = Stroke::new(e.width, self.settings_style.color_edge(self.p.ctx(), e));
        let shape_basic = CubicBezierShape::from_points_stroke(
            [
                pos_start_and_end,
                control_point1,
                control_point2,
                pos_start_and_end,
            ],
            true,
            Color32::TRANSPARENT,
            stroke,
        );

        if !comp_edge.subselected() {
            self.p.add(shape_basic);
            return vec![];
        }

        let mut shapes = vec![shape_basic];

        let highlighted_stroke = Stroke::new(
            e.width * 2.,
            self.settings_style.color_edge_highlight(comp_edge).unwrap(),
        );
        shapes.push(CubicBezierShape::from_points_stroke(
            [
                pos_start_and_end,
                control_point1,
                control_point2,
                pos_start_and_end,
            ],
            true,
            Color32::TRANSPARENT,
            highlighted_stroke,
        ));

        shapes
    }

    fn draw_edge_basic(
        &self,
        start_idx: &NodeIndex,
        end_idx: &NodeIndex,
        e: &Edge<E>,
        comp_edge: &StateComputedEdge,
        order: usize,
    ) -> (Vec<Shape>, Vec<QuadraticBezierShape>) {
        let mut comp_start = self.comp.node_state(start_idx).unwrap();
        let mut comp_end = self.comp.node_state(end_idx).unwrap();
        let mut start_node = self.g.node(*start_idx).unwrap();
        let mut end_node = self.g.node(*end_idx).unwrap();
        let mut transparent = false;

        if (start_node.folded || comp_start.subfolded()) && comp_end.subfolded() {
            return (vec![], vec![]);
        }

        if comp_start.subfolded() && !comp_end.subfolded() {
            let new_start_idx = self
                .comp
                .foldings
                .as_ref()
                .unwrap()
                .roots_by_node(*start_idx)
                .unwrap()
                .first()
                .unwrap();
            comp_start = self.comp.node_state(new_start_idx).unwrap();
            start_node = self.g.node(*new_start_idx).unwrap();
            transparent = true;
        }

        if !comp_start.subfolded() && comp_end.subfolded() {
            let new_end_idx = self
                .comp
                .foldings
                .as_ref()
                .unwrap()
                .roots_by_node(*end_idx)
                .unwrap()
                .first()
                .unwrap();
            comp_end = self.comp.node_state(new_end_idx).unwrap();
            end_node = self.g.node(*new_end_idx).unwrap();
            transparent = true;
        }

        let pos_start = start_node.screen_transform(self.meta).location.to_pos2();
        let pos_end = end_node.screen_transform(self.meta).location.to_pos2();

        let vec = pos_end - pos_start;
        let l = vec.length();
        let dir = vec / l;

        let start_node_radius_vec =
            Vec2::new(comp_start.radius(self.meta), comp_start.radius(self.meta)) * dir;
        let end_node_radius_vec =
            Vec2::new(comp_end.radius(self.meta), comp_end.radius(self.meta)) * dir;

        let tip_point = pos_start + vec - end_node_radius_vec;
        let start_point = pos_start + start_node_radius_vec;

        let mut color = self.settings_style.color_edge(self.p.ctx(), e);
        if transparent {
            color = color.gamma_multiply(0.15);
        }
        let stroke = Stroke::new(e.width, color);

        // draw straight edge
        if order == 0 {
            let mut shapes = vec![];
            let head_point_1 = tip_point - e.tip_size * rotate_vector(dir, e.tip_angle);
            let head_point_2 = tip_point - e.tip_size * rotate_vector(dir, -e.tip_angle);

            shapes.push(Shape::line_segment([start_point, tip_point], stroke));

            // draw tips for directed edges
            if self.g.is_directed() {
                shapes.push(Shape::line_segment([tip_point, head_point_1], stroke));
                shapes.push(Shape::line_segment([tip_point, head_point_2], stroke));
            }

            if !comp_edge.subselected() {
                shapes.into_iter().for_each(|shape| {
                    self.p.add(shape);
                });

                return (vec![], vec![]);
            }

            let highlighted_stroke = Stroke::new(
                e.width * 2.,
                self.settings_style.color_edge_highlight(comp_edge).unwrap(),
            );
            shapes.push(Shape::line_segment(
                [start_point, tip_point],
                highlighted_stroke,
            ));

            if self.g.is_directed() {
                shapes.push(Shape::line_segment(
                    [tip_point, head_point_1],
                    highlighted_stroke,
                ));
                shapes.push(Shape::line_segment(
                    [tip_point, head_point_2],
                    highlighted_stroke,
                ));
            }

            return (shapes, vec![]);
        }

        let mut shapes = vec![];
        let mut quadratic_shapes = vec![];
        let dir_perpendicular = Vec2::new(-dir.y, dir.x);
        let center_point = (start_point + tip_point.to_vec2()).to_vec2() / 2.0;
        let control_point =
            (center_point + dir_perpendicular * e.curve_size * order as f32).to_pos2();

        let tip_vec = control_point - tip_point;
        let tip_dir = tip_vec / tip_vec.length();
        let tip_size = e.tip_size;

        let arrow_tip_dir_1 = rotate_vector(tip_dir, e.tip_angle) * tip_size;
        let arrow_tip_dir_2 = rotate_vector(tip_dir, -e.tip_angle) * tip_size;

        let head_point_1 = tip_point + arrow_tip_dir_1;
        let head_point_2 = tip_point + arrow_tip_dir_2;

        quadratic_shapes.push(QuadraticBezierShape::from_points_stroke(
            [start_point, control_point, tip_point],
            false,
            Color32::TRANSPARENT,
            stroke,
        ));
        shapes.push(Shape::line_segment([tip_point, head_point_1], stroke));
        shapes.push(Shape::line_segment([tip_point, head_point_2], stroke));

        if !comp_edge.subselected() {
            quadratic_shapes.into_iter().for_each(|shape| {
                self.p.add(shape);
            });
            shapes.into_iter().for_each(|shape| {
                self.p.add(shape);
            });

            return (vec![], vec![]);
        }

        let highlighted_stroke = Stroke::new(
            e.width * 2.,
            self.settings_style.color_edge_highlight(comp_edge).unwrap(),
        );
        quadratic_shapes.push(QuadraticBezierShape::from_points_stroke(
            [start_point, control_point, tip_point],
            false,
            Color32::TRANSPARENT,
            highlighted_stroke,
        ));
        shapes.push(Shape::line_segment(
            [tip_point, head_point_1],
            highlighted_stroke,
        ));
        shapes.push(Shape::line_segment(
            [tip_point, head_point_2],
            highlighted_stroke,
        ));

        (shapes, quadratic_shapes)
    }

    fn draw_node(
        &self,
        n: &Node<N>,
        comp_node: &StateComputedNode,
    ) -> (Vec<CircleShape>, Vec<TextShape>) {
        let node = &n.screen_transform(self.meta);
        let loc = node.location.to_pos2();

        let (mut circles, mut texts) = self.draw_node_basic(loc, node, comp_node);
        let (circles_interacted, texts_interacted) =
            self.draw_node_interacted(loc, node, comp_node);
        circles.extend(circles_interacted);
        texts.extend(texts_interacted);
        (circles, texts)
    }

    fn draw_node_basic(
        &self,
        loc: Pos2,
        node: &Node<N>,
        comp_node: &StateComputedNode,
    ) -> (Vec<CircleShape>, Vec<TextShape>) {
        let color = self.settings_style.color_node(self.p.ctx(), node);
        let (mut nodes, mut texts) = (vec![], vec![]);
        if !(node.selected
            || comp_node.subselected()
            || node.dragged
            || node.folded
            || comp_node.subfolded())
        {
            // draw the node in place
            let node_radius = comp_node.radius(self.meta);
            self.p.circle_filled(
                loc,
                node_radius,
                self.settings_style.color_node(self.p.ctx(), node),
            );

            if let Some(label) = node.label.as_ref() {
                if !self.settings_style.labels_always {
                    return (nodes, texts);
                }

                let color_label = self.settings_style.color_label(self.p.ctx());
                let label_pos = Pos2::new(node.location.x, node.location.y - node_radius * 2.);
                let label_size = node_radius;
                let galley = self.p.layout_no_wrap(
                    label.clone(),
                    FontId::new(label_size, FontFamily::Monospace),
                    color_label,
                );
                self.p.add(TextShape::new(label_pos, galley));
            };

            return (nodes, texts);
        }

        if node.folded || comp_node.subfolded() {
            return (nodes, texts);
        }

        let node_radius = comp_node.radius(self.meta);
        nodes.push(CircleShape {
            center: loc,
            radius: node_radius,
            fill: color,
            stroke: Stroke::new(1., color),
        });

        if let Some(label) = node.label.as_ref() {
            if !(node.selected || comp_node.subselected()) && self.settings_style.labels_always {
                return (nodes, texts);
            }

            let color_label = self.settings_style.color_label(self.p.ctx());
            let label_pos = Pos2::new(node.location.x, node.location.y - node_radius * 2.);
            let label_size = node_radius;
            let galley = self.p.layout_no_wrap(
                label.clone(),
                FontId::new(label_size, FontFamily::Monospace),
                color_label,
            );
            texts.push(TextShape::new(label_pos, galley));
        };
        // draw the node later if it's selected or dragged to make sure it's on top
        (nodes, texts)
    }

    fn draw_node_interacted(
        &self,
        loc: Pos2,
        node: &Node<N>,
        comp_node: &StateComputedNode,
    ) -> (Vec<CircleShape>, Vec<TextShape>) {
        if !(node.selected
            || comp_node.subselected()
            || node.dragged
            || node.folded
            || comp_node.subfolded())
        {
            return (vec![], vec![]);
        }

        if comp_node.subfolded() {
            return (vec![], vec![]);
        }

        let mut circles = vec![];
        let mut texts = vec![];
        let node_radius = comp_node.radius(self.meta);
        let highlight_radius = node_radius * 1.5;
        let text_size = node_radius / 2.;
        let shape: CircleShape;

        if let Some(label) = node.label.as_ref() {
            let color_label = self.settings_style.color_label(self.p.ctx());
            let label_pos = Pos2::new(node.location.x, node.location.y - node_radius * 2.);
            let galley = self.p.layout_no_wrap(
                label.clone(),
                FontId::new(text_size, FontFamily::Monospace),
                color_label,
            );
            texts.push(TextShape::new(label_pos, galley));
        };

        if node.folded {
            shape = CircleShape::stroke(
                loc,
                node_radius,
                Stroke::new(1., self.settings_style.color_node(self.p.ctx(), node)),
            );
            texts.push(TextShape::new(
                Pos2::new(loc.x - node_radius / 4., loc.y - node_radius / 4.),
                self.p.layout_no_wrap(
                    comp_node.num_folded.to_string(),
                    FontId::monospace(text_size),
                    self.settings_style.color_label(self.p.ctx()),
                ),
            ));
        } else {
            shape = CircleShape {
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
        }

        circles.push(shape);
        (circles, texts)
    }
}

fn rotate_vector(vec: Vec2, angle: f32) -> Vec2 {
    let cos = angle.cos();
    let sin = angle.sin();
    Vec2::new(cos * vec.x - sin * vec.y, sin * vec.x + cos * vec.y)
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
}
