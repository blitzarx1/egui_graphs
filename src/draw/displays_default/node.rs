use egui::{
    epaint::{CircleShape, TextShape},
    Color32, FontFamily, FontId, Pos2, Shape, Stroke, Vec2,
};
use petgraph::{stable_graph::IndexType, EdgeType};

use crate::{draw::drawer::DrawContext, DisplayNode, NodeProps};

/// This is the default node shape which is used to display nodes in the graph.
///
/// You can use this implementation as an example for implementing your own custom node shapes.
#[derive(Clone, Debug)]
pub struct DefaultNodeShape {
    pub pos: Pos2,

    pub selected: bool,
    pub dragged: bool,
    pub color: Option<Color32>,

    pub label_text: String,

    /// Shape dependent property
    pub radius: f32,
}

impl<N: Clone> From<NodeProps<N>> for DefaultNodeShape {
    fn from(node_props: NodeProps<N>) -> Self {
        DefaultNodeShape {
            pos: node_props.location(),
            selected: node_props.selected,
            dragged: node_props.dragged,
            label_text: node_props.label.to_string(),
            color: node_props.color(),

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

        let style = if is_interacted {
            ctx.ctx.style().visuals.widgets.active
        } else {
            ctx.ctx.style().visuals.widgets.inactive
        };

        let color = if let Some(c) = self.color {
            c
        } else {
            style.fg_stroke.color
        };

        let circle_center = ctx.meta.canvas_to_screen_pos(self.pos);
        let circle_radius = ctx.meta.canvas_to_screen_size(self.radius);
        // Base stroke (currently unused in default impl, but exposed for hook customization)
        let base_stroke = Stroke::default();
        let stroke = if let Some(hook) = &ctx.style.node_stroke_hook {
            // Provide egui global style so user can rely on theme colors.
            let style_ref: &egui::Style = &*ctx.ctx.style();
            (hook)(
                self.selected,
                self.dragged,
                self.color,
                base_stroke,
                style_ref,
            )
        } else {
            base_stroke
        };

        let circle_shape = CircleShape {
            center: circle_center,
            radius: circle_radius,
            fill: color,
            stroke,
        };
        res.push(circle_shape.into());

        let label_visible = ctx.style.labels_always || self.selected || self.dragged;
        if !label_visible {
            return res;
        }

        let galley = ctx.ctx.fonts(|f| {
            f.layout_no_wrap(
                self.label_text.clone(),
                FontId::new(circle_radius, FontFamily::Monospace),
                color,
            )
        });

        // display label centered over the circle
        let label_pos = Pos2::new(
            circle_center.x - galley.size().x / 2.,
            circle_center.y - circle_radius * 2.,
        );

        let label_shape = TextShape::new(label_pos, galley, color);
        res.push(label_shape.into());

        res
    }

    fn update(&mut self, state: &NodeProps<N>) {
        self.pos = state.location();
        self.selected = state.selected;
        self.dragged = state.dragged;
        self.label_text = state.label.to_string();
        self.color = state.color();
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
