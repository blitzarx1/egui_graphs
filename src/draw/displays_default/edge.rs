use core::panic;

use egui::{
    epaint::{CubicBezierShape, TextShape},
    Color32, FontFamily, FontId, Pos2, Shape, Stroke, Vec2,
};
use petgraph::{stable_graph::IndexType, EdgeType};

use crate::{draw::DrawContext, elements::EdgeProps, node_size, DisplayEdge, DisplayNode, Node};

use super::edge_shape_builder::{EdgeShapeBuilder, TipProps};

#[derive(Clone, Debug)]
pub struct DefaultEdgeShape {
    pub order: usize,
    pub selected: bool,

    pub width: f32,
    pub tip_size: f32,
    pub tip_angle: f32,
    pub curve_size: f32,
    pub loop_size: f32,
    pub label_text: String,
}

impl<E: Clone> From<EdgeProps<E>> for DefaultEdgeShape {
    fn from(edge: EdgeProps<E>) -> Self {
        Self {
            order: edge.order,
            selected: edge.selected,
            label_text: edge.label,

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
            return self.is_inside_loop(start, pos);
        }

        if self.order == 0 {
            return self.is_inside_line(start, end, pos);
        }

        self.is_inside_curve(start, end, pos)
    }

    #[allow(clippy::too_many_lines)] // TODO: refactor
    fn shapes(
        &mut self,
        start: &Node<N, E, Ty, Ix, D>,
        end: &Node<N, E, Ty, Ix, D>,
        ctx: &DrawContext,
    ) -> Vec<egui::Shape> {
        let mut res = vec![];

        let label_visible = ctx.style.labels_always || self.selected;

        let style = if self.selected {
            ctx.ctx.style().visuals.widgets.active
        } else {
            ctx.ctx.style().visuals.widgets.inactive
        };
        let color = style.fg_stroke.color;
        let stroke = Stroke::new(self.width, color);

        if start.id() == end.id() {
            // draw loop
            let size = node_size(start, Vec2::new(-1., 0.));
            let mut line_looped_shapes = EdgeShapeBuilder::new(stroke)
                .looped(start.location(), size, self.loop_size, self.order)
                .with_scaler(ctx.meta)
                .build();
            let line_looped_shape = line_looped_shapes.clone().pop().unwrap();
            res.push(line_looped_shape);

            let Shape::CubicBezier(line_looped) = line_looped_shapes.pop().unwrap() else {
                panic!("invalid shape type")
            };

            // TODO: export to func
            if label_visible {
                let galley = ctx.ctx.fonts(|f| {
                    f.layout_no_wrap(
                        self.label_text.clone(),
                        FontId::new(ctx.meta.canvas_to_screen_size(size), FontFamily::Monospace),
                        color,
                    )
                });

                let flattened_curve = line_looped.flatten(None);
                let median = *flattened_curve.get(flattened_curve.len() / 2).unwrap();

                let label_width = galley.rect.width();
                let label_height = galley.rect.height();
                let pos = Pos2::new(median.x - label_width / 2., median.y - label_height);

                let label_shape = TextShape::new(pos, galley, color);
                res.push(label_shape.into());
            }
            return res;
        }

        let dir = (end.location() - start.location()).normalized();
        let start_connector_point = start.display().closest_boundary_point(dir);
        let end_connector_point = end.display().closest_boundary_point(-dir);

        if self.order == 0 {
            // draw straight edge

            let mut builder = EdgeShapeBuilder::new(stroke)
                .straight((start_connector_point, end_connector_point))
                .with_scaler(ctx.meta);

            let tip_props = TipProps {
                size: self.tip_size,
                angle: self.tip_angle,
            };
            if ctx.is_directed {
                builder = builder.with_tip(&tip_props);
            }
            let straight_shapes = builder.build();
            res.extend(straight_shapes);

            // TODO: export to func
            if label_visible {
                let size = f32::midpoint(node_size(start, dir), node_size(end, dir));
                let galley = ctx.ctx.fonts(|f| {
                    f.layout_no_wrap(
                        self.label_text.clone(),
                        FontId::new(ctx.meta.canvas_to_screen_size(size), FontFamily::Monospace),
                        color,
                    )
                });

                let dist = end_connector_point - start_connector_point;
                let center = ctx
                    .meta
                    .canvas_to_screen_pos(start_connector_point + dist / 2.);
                let label_width = galley.rect.width();
                let label_height = galley.rect.height();
                let pos = Pos2::new(center.x - label_width / 2., center.y - label_height);

                let label_shape = TextShape::new(pos, galley, color);
                res.push(label_shape.into());
            }

            return res;
        }

        let mut builder = EdgeShapeBuilder::new(stroke)
            .curved(
                (start_connector_point, end_connector_point),
                self.curve_size,
                self.order,
            )
            .with_scaler(ctx.meta);

        let tip_props = TipProps {
            size: self.tip_size,
            angle: self.tip_angle,
        };
        if ctx.is_directed {
            builder = builder.with_tip(&tip_props);
        }
        let curved_shapes = builder.build();
        let Some(Shape::CubicBezier(line_curved)) = curved_shapes.first() else {
            panic!("invalid shape type")
        };
        res.extend(curved_shapes.clone());

        if label_visible {
            let size = f32::midpoint(node_size(start, dir), node_size(end, dir));
            let galley = ctx.ctx.fonts(|f| {
                f.layout_no_wrap(
                    self.label_text.clone(),
                    FontId::new(ctx.meta.canvas_to_screen_size(size), FontFamily::Monospace),
                    color,
                )
            });

            let flattened_curve = line_curved.flatten(None);
            let median = *flattened_curve.get(flattened_curve.len() / 2).unwrap();

            let label_width = galley.rect.width();
            let label_height = galley.rect.height();
            let pos = Pos2::new(median.x - label_width / 2., median.y - label_height);

            let label_shape = TextShape::new(pos, galley, color);
            res.push(label_shape.into());
        }

        res
    }

    fn update(&mut self, state: &EdgeProps<E>) {
        self.order = state.order;
        self.selected = state.selected;
        self.label_text = state.label.to_string();
    }
}

impl DefaultEdgeShape {
    fn is_inside_loop<
        E: Clone,
        N: Clone,
        Ix: IndexType,
        Ty: EdgeType,
        D: DisplayNode<N, E, Ty, Ix>,
    >(
        &self,
        node: &Node<N, E, Ty, Ix, D>,
        pos: Pos2,
    ) -> bool {
        let node_size = node_size(node, Vec2::new(-1., 0.));

        let shape = EdgeShapeBuilder::new(Stroke::new(self.width, Color32::default()))
            .looped(node.location(), node_size, self.loop_size, self.order)
            .build();

        match shape.first() {
            Some(Shape::CubicBezier(cubic)) => is_point_on_curve(pos, cubic, self.width),
            _ => panic!("invalid shape type"),
        }
    }

    fn is_inside_line<
        E: Clone,
        N: Clone,
        Ix: IndexType,
        Ty: EdgeType,
        D: DisplayNode<N, E, Ty, Ix>,
    >(
        &self,
        start: &Node<N, E, Ty, Ix, D>,
        end: &Node<N, E, Ty, Ix, D>,
        pos: Pos2,
    ) -> bool {
        distance_segment_to_point(start.location(), end.location(), pos) <= self.width
    }

    fn is_inside_curve<
        N: Clone,
        E: Clone,
        Ty: EdgeType,
        Ix: IndexType,
        D: DisplayNode<N, E, Ty, Ix>,
    >(
        &self,
        node_start: &Node<N, E, Ty, Ix, D>,
        node_end: &Node<N, E, Ty, Ix, D>,
        pos: Pos2,
    ) -> bool {
        let dir = (node_end.location() - node_start.location()).normalized();
        let start = node_start.display().closest_boundary_point(dir);
        let end = node_end.display().closest_boundary_point(-dir);

        let stroke = Stroke::new(self.width, Color32::default());
        let curved_shapes = EdgeShapeBuilder::new(stroke)
            .curved((start, end), self.curve_size, self.order)
            .build();

        let curved_shape = match curved_shapes.first() {
            Some(Shape::CubicBezier(curve)) => curve.clone(),
            _ => panic!("invalid shape type"),
        };
        is_point_on_curve(pos, &curved_shape, self.width)
    }
}

/// Returns the distance from line segment [`a`, `b`] to point `c`.
/// Adapted from <https://stackoverflow.com/questions/1073336/circle-line-segment-collision-detection-algorithm>
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

fn is_point_on_curve(point: Pos2, curve: &CubicBezierShape, tolerance: f32) -> bool {
    for p in curve.flatten(None) {
        if p.distance(point) < tolerance {
            return true;
        }
    }
    false
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
}
