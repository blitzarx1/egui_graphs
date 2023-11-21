use std::time::Instant;

use egui::{
    emath::Rot2, epaint::TextShape, Color32, FontFamily, FontId, Pos2, Rect, Shape, Stroke, Vec2,
};
use egui_graphs::{DisplayNode, NodeProps};
use petgraph::{stable_graph::IndexType, EdgeType};

/// Rotates node whent the node is being dragged.
#[derive(Clone)]
pub struct NodeShapeAnimated {
    label: String,
    loc: Pos2,
    dragged: bool,

    angle_rad: f32,
    speed_per_second: f32,
    /// None means animation is not in progress
    last_time_update: Option<Instant>,

    size: f32,
}

impl<N: Clone> From<NodeProps<N>> for NodeShapeAnimated {
    fn from(node_props: NodeProps<N>) -> Self {
        Self {
            label: node_props.label,
            loc: node_props.location,
            dragged: node_props.dragged,

            angle_rad: Default::default(),
            last_time_update: Default::default(),
            speed_per_second: 1.,

            size: 30.,
        }
    }
}

impl<N: Clone, E: Clone, Ty: EdgeType, Ix: IndexType> DisplayNode<N, E, Ty, Ix>
    for NodeShapeAnimated
{
    fn is_inside(&self, pos: Pos2) -> bool {
        let rotated_pos = rotate_point_around(self.loc, pos, -self.angle_rad);
        let rect = Rect::from_center_size(self.loc, Vec2::new(self.size, self.size));

        rect.contains(rotated_pos)
    }

    fn closest_boundary_point(&self, dir: Vec2) -> Pos2 {
        let rotated_dir = rotate_vector(dir, -self.angle_rad);
        let intersection_point = find_intersection(self.loc, self.size, rotated_dir);
        rotate_point_around(self.loc, intersection_point, self.angle_rad)
    }

    fn shapes(&mut self, ctx: &egui_graphs::DrawContext) -> Vec<egui::Shape> {
        // lets draw a rect with label in the center for every node
        // which rotates when the node is dragged

        // find node center location on the screen coordinates
        let center = ctx.meta.canvas_to_screen_pos(self.loc);
        let size = ctx.meta.canvas_to_screen_size(self.size);
        let rect_default = Rect::from_center_size(center, Vec2::new(size, size));
        let color = ctx.ctx.style().visuals.weak_text_color();

        let diff = match self.dragged {
            true => {
                let now = Instant::now();
                match self.last_time_update {
                    Some(last_time) => {
                        self.last_time_update = Some(now);
                        let seconds_passed = now.duration_since(last_time);
                        seconds_passed.as_secs_f32() * self.speed_per_second
                    }
                    None => {
                        self.last_time_update = Some(now);
                        0.
                    }
                }
            }
            false => {
                if self.last_time_update.is_some() {
                    self.last_time_update = None;
                }
                0.
            }
        };

        if diff > 0. {
            let curr_angle = self.angle_rad + diff;
            let rot = Rot2::from_angle(curr_angle).normalized();
            self.angle_rad = rot.angle();
        };

        let points = rect_to_points(rect_default)
            .into_iter()
            .map(|p| rotate_point_around(center, p, self.angle_rad))
            .collect::<Vec<_>>();

        let shape_rect = Shape::convex_polygon(points, Color32::default(), Stroke::new(1., color));

        // create label
        let color = ctx.ctx.style().visuals.text_color();
        let galley = ctx.ctx.fonts(|f| {
            f.layout_no_wrap(
                self.label.clone(),
                FontId::new(ctx.meta.canvas_to_screen_size(10.), FontFamily::Monospace),
                color,
            )
        });

        // we need to offset label by half its size to place it in the center of the rect
        let offset = Vec2::new(-galley.size().x / 2., -galley.size().y / 2.);

        // create the shape and add it to the layers
        let shape_label = TextShape::new(center + offset, galley);

        vec![shape_rect, shape_label.into()]
    }

    fn update(&mut self, state: &NodeProps<N>) {
        self.label = state.label.clone();
        self.loc = state.location;
        self.dragged = state.dragged;
    }
}

fn find_intersection(center: Pos2, size: f32, direction: Vec2) -> Pos2 {
    // Determine the intersection side based on the direction
    if direction.x.abs() > direction.y.abs() {
        // Intersects left or right side
        let x = if direction.x > 0.0 {
            center.x + size / 2.0
        } else {
            center.x - size / 2.0
        };
        let y = center.y + direction.y / direction.x * (x - center.x);
        Pos2::new(x, y)
    } else {
        // Intersects top or bottom side
        let y = if direction.y > 0.0 {
            center.y + size / 2.0
        } else {
            center.y - size / 2.0
        };
        let x = center.x + direction.x / direction.y * (y - center.y);
        Pos2::new(x, y)
    }
}

// Function to rotate a point around another point
fn rotate_point_around(center: Pos2, point: Pos2, angle: f32) -> Pos2 {
    let sin_angle = angle.sin();
    let cos_angle = angle.cos();

    // Translate point back to origin
    let translated_point = point - center;

    // Rotate point
    let rotated_x = translated_point.x * cos_angle - translated_point.y * sin_angle;
    let rotated_y = translated_point.x * sin_angle + translated_point.y * cos_angle;

    // Translate point back
    Pos2::new(rotated_x, rotated_y) + center.to_vec2()
}

fn rect_to_points(rect: Rect) -> Vec<Pos2> {
    let top_left = rect.min;
    let bottom_right = rect.max;

    // Calculate the other two corners
    let top_right = Pos2::new(bottom_right.x, top_left.y);
    let bottom_left = Pos2::new(top_left.x, bottom_right.y);

    vec![top_left, top_right, bottom_right, bottom_left]
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
    fn test_intersection_right_side() {
        let center = Pos2::new(0.0, 0.0);
        let size = 10.;
        let direction = Vec2::new(1.0, 0.0); // Directly to the right
        let expected = Pos2::new(5.0, 0.0);
        assert_eq!(find_intersection(center, size, direction), expected);
    }

    #[test]
    fn test_intersection_top_side() {
        let center = Pos2::new(0.0, 0.0);
        let size = 10.;
        let direction = Vec2::new(0.0, 1.0); // Directly upwards
        let expected = Pos2::new(0.0, 5.0);
        assert_eq!(find_intersection(center, size, direction), expected);
    }
}
