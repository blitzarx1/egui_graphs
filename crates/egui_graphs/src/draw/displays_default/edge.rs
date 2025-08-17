use core::panic;

use egui::{
    epaint::{CubicBezierShape, TextShape},
    Color32, FontFamily, FontId, Pos2, Shape, Stroke, Vec2,
};
use petgraph::{stable_graph::IndexType, EdgeType};

use crate::{draw::DrawContext, elements::EdgeProps, node_size, DisplayEdge, DisplayNode, Node};

use super::edge_shape_builder::{EdgeShapeBuilder, TipProps};

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
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

    fn shapes(
        &mut self,
        start: &Node<N, E, Ty, Ix, D>,
        end: &Node<N, E, Ty, Ix, D>,
        ctx: &DrawContext,
    ) -> Vec<egui::Shape> {
        let label_visible = ctx.style.labels_always || self.selected;
        let color = self.current_color(ctx);
        let stroke = self.current_stroke(ctx, color);

        if start.id() == end.id() {
            return self.loop_shapes(start, ctx, stroke, color, label_visible);
        }

        let dir = (end.location() - start.location()).normalized();
        if self.order == 0 {
            return self.straight_shapes(start, end, ctx, dir);
        }
        self.curved_shapes(start, end, ctx, dir)
    }

    fn update(&mut self, state: &EdgeProps<E>) {
        self.order = state.order;
        self.selected = state.selected;
        self.label_text = state.label.to_string();
    }

    fn extra_bounds(
        &self,
        start: &Node<N, E, Ty, Ix, D>,
        end: &Node<N, E, Ty, Ix, D>,
    ) -> Option<(Pos2, Pos2)> {
        use crate::helpers::node_size;
        // self-loop: approximate loop rectangle
        if start.id() == end.id() {
            let node_radius = node_size(start, Vec2::new(-1., 0.));
            let order = self.order as f32;
            let loop_radius = node_radius * (self.loop_size + order);
            let c = start.location();
            let min = Pos2::new(c.x - loop_radius, c.y - loop_radius);
            // bottom extent does not go below node center + radius, existing node bounds already include that
            let max = Pos2::new(c.x + loop_radius, c.y + node_radius);
            return Some((min, max));
        }

        // curved edges (order > 0): approximate cubic bezier hull from control points
        if self.order > 0 {
            let dir_vec = end.location() - start.location();
            if dir_vec == Vec2::ZERO {
                return None;
            }
            // connector points
            let dir = dir_vec.normalized();
            let start_p = start.display().closest_boundary_point(dir);
            let end_p = end.display().closest_boundary_point(-dir);
            let dist = end_p - start_p;
            if dist == Vec2::ZERO {
                return None;
            }
            let dir_n = dist.normalized();
            let dir_perp = Vec2::new(-dir_n.y, dir_n.x);
            let center_point = start_p + dist / 2.0;
            let param = self.order as f32;
            let height = dir_perp * self.curve_size * param;
            let cp_center = center_point + height;
            // replicate control points logic approximately
            // avoid division by zero in pathological cases
            let denom = param * dist * 0.5;
            let mut adjust = Vec2::ZERO;
            if denom.x != 0.0 && denom.y != 0.0 {
                adjust = dir_n * self.curve_size / denom;
            }
            let cp_start = cp_center - adjust;
            let cp_end = cp_center + adjust;

            let xs = [start_p.x, end_p.x, cp_start.x, cp_end.x, cp_center.x];
            let ys = [start_p.y, end_p.y, cp_start.y, cp_end.y, cp_center.y];
            let (min_x, max_x) = xs
                .iter()
                .fold((f32::MAX, f32::MIN), |(mi, ma), v| (mi.min(*v), ma.max(*v)));
            let (min_y, max_y) = ys
                .iter()
                .fold((f32::MAX, f32::MIN), |(mi, ma), v| (mi.min(*v), ma.max(*v)));
            return Some((Pos2::new(min_x, min_y), Pos2::new(max_x, max_y)));
        }

        None
    }
}

impl DefaultEdgeShape {
    fn current_color(&self, ctx: &DrawContext) -> Color32 {
        let style = if self.selected {
            ctx.ctx.style().visuals.widgets.active
        } else {
            ctx.ctx.style().visuals.widgets.inactive
        };
        style.fg_stroke.color
    }

    fn current_stroke(&self, ctx: &DrawContext, color: Color32) -> Stroke {
        let base = Stroke::new(self.width, color);
        if let Some(hook) = &ctx.style.edge_stroke_hook {
            let style_ref: &egui::Style = &ctx.ctx.style();
            (hook)(self.selected, self.order, base, style_ref)
        } else {
            base
        }
    }

    fn loop_shapes<
        N: Clone,
        E: Clone,
        Ty: EdgeType,
        Ix: IndexType,
        D: DisplayNode<N, E, Ty, Ix>,
    >(
        &mut self,
        start: &Node<N, E, Ty, Ix, D>,
        ctx: &DrawContext,
        stroke: Stroke,
        color: Color32,
        label_visible: bool,
    ) -> Vec<Shape> {
        let mut res = vec![];
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
        if label_visible {
            let galley = ctx.ctx.fonts(|f| {
                f.layout_no_wrap(
                    self.label_text.clone(),
                    FontId::new(ctx.meta.canvas_to_screen_size(size), FontFamily::Monospace),
                    color,
                )
            });
            let median = Self::median_point(&line_looped);
            res.push(Self::label_shape(galley, median, color));
        }
        res
    }

    fn straight_shapes<
        N: Clone,
        E: Clone,
        Ty: EdgeType,
        Ix: IndexType,
        D: DisplayNode<N, E, Ty, Ix>,
    >(
        &mut self,
        start: &Node<N, E, Ty, Ix, D>,
        end: &Node<N, E, Ty, Ix, D>,
        ctx: &DrawContext,
        dir: Vec2,
    ) -> Vec<Shape> {
        let mut res = vec![];
        let color = self.current_color(ctx);
        let stroke = self.current_stroke(ctx, color);
        let label_visible = ctx.style.labels_always || self.selected;
        let start_connector_point = start.display().closest_boundary_point(dir);
        let end_connector_point = end.display().closest_boundary_point(-dir);
        let mut builder = EdgeShapeBuilder::new(stroke)
            .straight((start_connector_point, end_connector_point))
            .with_scaler(ctx.meta);
        let mut tip_store: Option<TipProps> = None;
        if ctx.is_directed {
            tip_store = Some(TipProps {
                size: self.tip_size,
                angle: self.tip_angle,
            });
        }
        if let Some(ref tip) = tip_store {
            builder = builder.with_tip(tip);
        }
        let straight_shapes = builder.build();
        res.extend(straight_shapes);
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
            res.push(Self::label_shape(galley, center, color));
        }
        res
    }

    fn curved_shapes<
        N: Clone,
        E: Clone,
        Ty: EdgeType,
        Ix: IndexType,
        D: DisplayNode<N, E, Ty, Ix>,
    >(
        &mut self,
        start: &Node<N, E, Ty, Ix, D>,
        end: &Node<N, E, Ty, Ix, D>,
        ctx: &DrawContext,
        dir: Vec2,
    ) -> Vec<Shape> {
        let mut res = vec![];
        let color = self.current_color(ctx);
        let stroke = self.current_stroke(ctx, color);
        let label_visible = ctx.style.labels_always || self.selected;
        let start_connector_point = start.display().closest_boundary_point(dir);
        let end_connector_point = end.display().closest_boundary_point(-dir);
        let mut builder = EdgeShapeBuilder::new(stroke)
            .curved(
                (start_connector_point, end_connector_point),
                self.curve_size,
                self.order,
            )
            .with_scaler(ctx.meta);
        let mut tip_store: Option<TipProps> = None;
        if ctx.is_directed {
            tip_store = Some(TipProps {
                size: self.tip_size,
                angle: self.tip_angle,
            });
        }
        if let Some(ref tip) = tip_store {
            builder = builder.with_tip(tip);
        }
        let curved_shapes = builder.build();
        // Use first shape for label anchor. It may be a cubic or a straight segment (degenerate case).
        if let Some(first) = curved_shapes.first() {
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
                let anchor = match first {
                    Shape::CubicBezier(cubic) => Self::median_point(cubic),
                    Shape::LineSegment { points, .. } => {
                        let mid =
                            (*points.first().unwrap() + points.last().unwrap().to_vec2()) / 2.0;
                        mid
                    }
                    _ => {
                        // Fallback to midpoint between connectors in canvas coords transformed to screen
                        let center_canvas = ((start_connector_point.to_vec2()
                            + end_connector_point.to_vec2())
                            / 2.0)
                            .to_pos2();
                        ctx.meta.canvas_to_screen_pos(center_canvas)
                    }
                };
                res.push(Self::label_shape(galley, anchor, color));
            }
        }
        res
    }

    fn label_shape(galley: std::sync::Arc<egui::Galley>, anchor: Pos2, color: Color32) -> Shape {
        let label_width = galley.rect.width();
        let label_height = galley.rect.height();
        let pos = Pos2::new(anchor.x - label_width / 2., anchor.y - label_height);
        TextShape::new(pos, galley, color).into()
    }

    fn median_point(curve: &CubicBezierShape) -> Pos2 {
        // Ensure positive tolerance to avoid epaint panic on some platforms/configs.
        let flattened = curve.flatten(Some(1.0_f32));
        *flattened.get(flattened.len() / 2).unwrap()
    }

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

        let loop_stroke = Stroke::new(self.width, Color32::default());
        let shape = EdgeShapeBuilder::new(loop_stroke)
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
    // Positive tessellation tolerance for robust flattening.
    for p in curve.flatten(Some(1.0_f32)) {
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

    #[test]
    fn test_median_point_no_panic() {
        let stroke = Stroke::new(1.0, Color32::WHITE);
        let curve = CubicBezierShape::from_points_stroke(
            [
                Pos2::new(0.0, 0.0),
                Pos2::new(5.0, 10.0),
                Pos2::new(10.0, 10.0),
                Pos2::new(10.0, 0.0),
            ],
            false,
            Color32::TRANSPARENT,
            stroke,
        );
        let _ = DefaultEdgeShape::median_point(&curve);
    }

    #[test]
    fn test_is_point_on_curve_positive_tolerance() {
        let stroke = Stroke::new(1.0, Color32::WHITE);
        let curve = CubicBezierShape::from_points_stroke(
            [
                Pos2::new(0.0, 0.0),
                Pos2::new(5.0, 10.0),
                Pos2::new(10.0, 10.0),
                Pos2::new(10.0, 0.0),
            ],
            false,
            Color32::TRANSPARENT,
            stroke,
        );
        let _ = is_point_on_curve(Pos2::new(5.0, 5.0), &curve, 2.0);
    }
}
