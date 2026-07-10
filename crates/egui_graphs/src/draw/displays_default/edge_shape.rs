use std::f32::consts::PI;

use egui::{epaint::CubicBezierShape, Color32, Pos2, Shape, Stroke, Vec2};

use crate::metadata::MetadataFrame;

/// Geometry used by [`EdgeShapeBuilder`] to construct an edge body.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum EdgeShapeProps {
    Straight {
        bounds: (Pos2, Pos2),
    },
    Curved {
        bounds: (Pos2, Pos2),
        curve_size: f32,
        order: usize,
    },
    Looped {
        node_center: Pos2,
        node_size: f32,
        loop_size: f32,
        order: usize,
    },
}

/// Arrow-tip geometry for a directed edge.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct TipProps {
    pub size: f32,
    pub angle: f32,
}

pub struct EdgeShapeBuilder<'a> {
    shape_props: EdgeShapeProps,
    tip: Option<TipProps>,
    stroke: Stroke,
    scaler: Option<&'a MetadataFrame>, // TODO: do we need metadata dependency here?
}

impl<'a> EdgeShapeBuilder<'a> {
    /// Creates a builder for the provided edge geometry and stroke.
    pub fn new(shape_props: EdgeShapeProps, stroke: Stroke) -> Self {
        Self {
            shape_props,
            stroke,
            tip: None,
            scaler: None,
        }
    }

    pub fn with_scaler(mut self, scaler: &'a MetadataFrame) -> Self {
        self.scaler = Some(scaler);

        self
    }

    pub fn with_tip(mut self, tip_props: TipProps) -> Self {
        self.tip = Some(tip_props);

        self
    }

    fn shape_straight(&self, bounds: (Pos2, Pos2)) -> Vec<Shape> {
        let mut res = vec![];

        let (start, end) = bounds;
        let mut stroke = self.stroke;

        let mut points_line = vec![start, end];
        let mut points_tip = match self.tip {
            Some(tip_props) => {
                let tip_dir = (end - start).normalized();

                let arrow_tip_dir_1 = rotate_vector(tip_dir, tip_props.angle) * tip_props.size;
                let arrow_tip_dir_2 = rotate_vector(tip_dir, -tip_props.angle) * tip_props.size;

                let tip_start_1 = end - arrow_tip_dir_1;
                let tip_start_2 = end - arrow_tip_dir_2;

                // replace end of an edge with start of tip
                *points_line.get_mut(1).unwrap() = end - tip_props.size * tip_dir;

                vec![end, tip_start_1, tip_start_2]
            }
            None => vec![],
        };

        if let Some(scaler) = self.scaler {
            stroke.width = scaler.canvas_to_screen_size(stroke.width);
            points_line = points_line
                .iter()
                .map(|p| scaler.canvas_to_screen_pos(*p))
                .collect();
            points_tip = points_tip
                .iter()
                .map(|p| scaler.canvas_to_screen_pos(*p))
                .collect();
        }

        res.push(Shape::line_segment(
            [points_line[0], points_line[1]],
            stroke,
        ));
        if !points_tip.is_empty() {
            res.push(Shape::convex_polygon(
                points_tip,
                stroke.color,
                Stroke::default(),
            ));
        }

        res
    }

    fn shape_looped(
        &self,
        node_center: Pos2,
        node_size: f32,
        loop_size: f32,
        param: f32,
    ) -> Vec<Shape> {
        let mut res = vec![];

        let mut stroke = self.stroke;
        let center_horizon_angle = PI / 4.;
        let y_intersect = node_center.y - node_size * center_horizon_angle.sin();

        let mut edge_start = Pos2::new(
            node_center.x - node_size * center_horizon_angle.cos(),
            y_intersect,
        );
        let mut edge_end = Pos2::new(
            node_center.x + node_size * center_horizon_angle.cos(),
            y_intersect,
        );

        let loop_size = node_size * (loop_size + param);

        let mut control_point1 = Pos2::new(node_center.x + loop_size, node_center.y - loop_size);
        let mut control_point2 = Pos2::new(node_center.x - loop_size, node_center.y - loop_size);

        if let Some(scaler) = self.scaler {
            stroke.width = scaler.canvas_to_screen_size(stroke.width);
            edge_end = scaler.canvas_to_screen_pos(edge_end);
            control_point1 = scaler.canvas_to_screen_pos(control_point1);
            control_point2 = scaler.canvas_to_screen_pos(control_point2);
            edge_start = scaler.canvas_to_screen_pos(edge_start);
        }

        res.push(
            CubicBezierShape::from_points_stroke(
                [edge_end, control_point1, control_point2, edge_start],
                false,
                Color32::default(),
                stroke,
            )
            .into(),
        );
        res
    }

    fn shape_curved(&self, bounds: (Pos2, Pos2), curve_size: f32, param: f32) -> Vec<Shape> {
        let mut res = vec![];
        let (start, end) = bounds;
        let mut stroke = self.stroke;

        let dist = end - start;
        let len = dist.length();

        // Guard against degenerate or overlapping nodes: fallback to straight if too short.
        if !len.is_finite() || len <= f32::EPSILON {
            return self.shape_straight((start, end));
        }

        let dir = dist / len; // safe: len > 0
        let dir_p = Vec2::new(-dir.y, dir.x);
        // Normal offset height controls bulge; independent of length for consistent look
        let offset = dir_p * (curve_size * param);
        // Place control points along the tangents for a smooth cubic
        let s = (len / 3.0).max(1.0);
        let cp_start = start + dir * s + offset;
        let cp_end = end - dir * s + offset;

        let mut points_curve = vec![start, cp_start, cp_end, end];

        let mut points_tip = match self.tip {
            Some(tip_props) => {
                let tip_dir = (end - cp_end).normalized();

                let arrow_tip_dir_1 = rotate_vector(tip_dir, tip_props.angle) * tip_props.size;
                let arrow_tip_dir_2 = rotate_vector(tip_dir, -tip_props.angle) * tip_props.size;

                let tip_start_1 = end - arrow_tip_dir_1;
                let tip_start_2 = end - arrow_tip_dir_2;

                // replace end of an edge with start of tip
                *points_curve.get_mut(3).unwrap() = end - tip_props.size * tip_dir;

                vec![end, tip_start_1, tip_start_2]
            }
            None => vec![],
        };

        if let Some(scaler) = self.scaler {
            stroke.width = scaler.canvas_to_screen_size(stroke.width);
            points_curve = points_curve
                .iter()
                .map(|p| scaler.canvas_to_screen_pos(*p))
                .collect();
            points_tip = points_tip
                .iter()
                .map(|p| scaler.canvas_to_screen_pos(*p))
                .collect();
        }

        res.push(
            CubicBezierShape::from_points_stroke(
                [
                    points_curve[0],
                    points_curve[1],
                    points_curve[2],
                    points_curve[3],
                ],
                false,
                Color32::default(),
                stroke,
            )
            .into(),
        );
        if !points_tip.is_empty() {
            res.push(Shape::convex_polygon(
                points_tip,
                stroke.color,
                Stroke::default(),
            ));
        }

        res
    }

    pub fn build(self) -> EdgeShape {
        match self.shape_props {
            EdgeShapeProps::Straight { bounds } => EdgeShape {
                shapes: self.shape_straight(bounds),
            },
            EdgeShapeProps::Looped {
                node_center,
                node_size,
                loop_size,
                order,
            } => {
                let param: f32 = order as f32;
                EdgeShape {
                    shapes: self.shape_looped(node_center, node_size, loop_size, param),
                }
            }
            EdgeShapeProps::Curved {
                bounds,
                curve_size,
                order,
            } => {
                let param: f32 = order as f32;
                EdgeShape {
                    shapes: self.shape_curved(bounds, curve_size, param),
                }
            }
        }
    }
}

/// Shapes produced for one edge, including its body and optional arrow tip.
#[derive(Clone, Debug)]
pub struct EdgeShape {
    shapes: Vec<Shape>,
}

impl EdgeShape {
    pub fn body(&self) -> &Shape {
        self.shapes
            .first()
            .expect("EdgeShape should have at least one shape")
    }

    /// Returns all shapes, with the edge body first and the optional arrow tip second.
    pub fn shapes(&self) -> &[Shape] {
        &self.shapes
    }

    /// Consumes this value and returns its shapes without cloning them.
    pub fn into_shapes(self) -> Vec<Shape> {
        self.shapes
    }
}

/// rotates vector by angle
fn rotate_vector(vec: Vec2, angle: f32) -> Vec2 {
    let cos = angle.cos();
    let sin = angle.sin();
    Vec2::new(cos * vec.x - sin * vec.y, sin * vec.x + cos * vec.y)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn curved_falls_back_to_straight_when_zero_length() {
        let stroke = Stroke::new(1.0, Color32::WHITE);
        let builder = EdgeShapeBuilder::new(
            EdgeShapeProps::Curved {
                bounds: (Pos2::new(0.0, 0.0), Pos2::new(0.0, 0.0)),
                curve_size: 20.0,
                order: 1,
            },
            stroke,
        );
        let shapes = builder.build();
        // Expect a straight line segment (fallback)
        assert!(matches!(shapes.body(), Shape::LineSegment { .. }));
    }

    #[test]
    fn curved_builds_cubic_for_normal_bounds() {
        let stroke = Stroke::new(1.0, Color32::WHITE);
        let builder = EdgeShapeBuilder::new(
            EdgeShapeProps::Curved {
                bounds: (Pos2::new(0.0, 0.0), Pos2::new(10.0, 0.0)),
                curve_size: 20.0,
                order: 1,
            },
            stroke,
        );
        let shapes = builder.build();
        // Body shape should be a cubic bezier
        assert!(matches!(shapes.body(), Shape::CubicBezier(_)));
    }
}
