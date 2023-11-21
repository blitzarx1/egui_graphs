use egui::{
    epaint::{CircleShape, TextShape},
    FontFamily, FontId, Pos2, Shape, Stroke, Vec2,
};
use petgraph::{stable_graph::IndexType, EdgeType};

use crate::{draw::drawer::DrawContext, NodeProps};

use super::DisplayNode;

/// This is the default node shape which is used to display nodes in the graph.
///
/// You can use this implementation as an example for implementing your own custom node shapes.
#[derive(Clone, Debug)]
pub struct DefaultNodeShape {
    pub pos: Pos2,

    pub selected: bool,
    pub dragged: bool,

    pub label_text: String,

    /// Shape defined property
    pub radius: f32,
}

impl From<NodeProps> for DefaultNodeShape {
    fn from(node_props: NodeProps) -> Self {
        DefaultNodeShape {
            pos: node_props.location,
            selected: node_props.selected,
            dragged: node_props.dragged,
            label_text: node_props.label.to_string(),

            radius: 5.0,
        }
    }
}

impl<N: Clone, E: Clone, Ty: EdgeType, Ix: IndexType> DisplayNode<N, E, Ty, Ix>
    for DefaultNodeShape
{
    fn is_inside(&self, pos: Pos2) -> bool {
        is_inside_circle(self.pos, self.radius, pos)
    }

    fn closest_boundary_point(&self, dir: Vec2) -> Pos2 {
        closest_point_on_circle(self.pos, self.radius, dir)
    }

    fn shapes(&mut self, ctx: &DrawContext) -> Vec<Shape> {
        let mut res = Vec::with_capacity(2);

        let is_interacted = self.selected || self.dragged;

        let style = match is_interacted {
            true => ctx.ctx.style().visuals.widgets.active,
            false => ctx.ctx.style().visuals.widgets.inactive,
        };
        let color = style.fg_stroke.color;

        let circle_center = ctx.meta.canvas_to_screen_pos(self.pos);
        let circle_radius = ctx.meta.canvas_to_screen_size(self.radius);
        let circle_shape = CircleShape {
            center: circle_center,
            radius: circle_radius,
            fill: color,
            stroke: Stroke::default(),
        };
        res.push(circle_shape.into());

        let label_visible = ctx.style.labels_always || self.selected || self.dragged;
        if !label_visible {
            return res;
        }

        // display label centered over the circle
        let label_pos = Pos2::new(circle_center.x, circle_center.y - circle_radius * 2.);
        let galley = ctx.ctx.fonts(|f| {
            f.layout_no_wrap(
                self.label_text.clone(),
                FontId::new(circle_radius, FontFamily::Monospace),
                color,
            )
        });

        let label_shape = TextShape::new(label_pos, galley);
        res.push(label_shape.into());

        res
    }

    fn update(&mut self, state: &NodeProps) {
        self.pos = state.location;
        self.pos = state.location;
        self.selected = state.selected;
        self.dragged = state.dragged;
        self.label_text = state.label.to_string();
    }
}

fn closest_point_on_circle(center: Pos2, radius: f32, dir: Vec2) -> Pos2 {
    center + dir.normalized() * radius
}

fn is_inside_circle(center: Pos2, radius: f32, pos: Pos2) -> bool {
    let dir = pos - center;
    dir.length() <= radius
}

#[cfg(test)]
mod test {
    use super::*;
    use egui::Pos2;

    #[test]
    fn test_closest_point_on_circle() {
        assert_eq!(
            closest_point_on_circle(Pos2::new(0.0, 0.0), 10.0, Vec2::new(5.0, 0.0)),
            Pos2::new(10.0, 0.0)
        );
        assert_eq!(
            closest_point_on_circle(Pos2::new(0.0, 0.0), 10.0, Vec2::new(15.0, 0.0)),
            Pos2::new(10.0, 0.0)
        );
        assert_eq!(
            closest_point_on_circle(Pos2::new(0.0, 0.0), 10.0, Vec2::new(0.0, 10.0)),
            Pos2::new(0.0, 10.0)
        );
    }

    #[test]
    fn test_is_inside_circle() {
        assert!(is_inside_circle(
            Pos2::new(0.0, 0.0),
            10.0,
            Pos2::new(5.0, 0.0)
        ));
        assert!(!is_inside_circle(
            Pos2::new(0.0, 0.0),
            10.0,
            Pos2::new(15.0, 0.0)
        ));
        assert!(is_inside_circle(
            Pos2::new(0.0, 0.0),
            10.0,
            Pos2::new(0.0, 10.0)
        ));
    }
}
