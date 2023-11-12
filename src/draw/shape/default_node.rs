use egui::{
    epaint::{CircleShape, TextShape},
    FontFamily, FontId, Pos2, Shape,
};
use petgraph::{stable_graph::IndexType, EdgeType};

use crate::{draw::custom::DrawContext, Node};

use super::{Interactable, NodeGraphDisplay};

#[derive(Clone, Debug)]
pub struct DefaultNodeShape {
    pub pos: Pos2,

    pub radius: f32,
    pub selected: bool,
    pub dragged: bool,

    pub label_text: String,
}

impl Interactable for DefaultNodeShape {
    fn is_inside(&self, pos: Pos2) -> bool {
        self.pos.distance(pos) < self.radius
    }
}

impl<N: Clone, E: Clone, Ty: EdgeType, Ix: IndexType> NodeGraphDisplay<N, E, Ty, Ix>
    for DefaultNodeShape
{
    fn from_node(node: &Node<N>) -> Self {
        DefaultNodeShape {
            pos: node.location().to_pos2(),

            radius: node.radius(),
            selected: node.selected(),
            dragged: node.dragged(),

            label_text: node.label().to_string(),
        }
    }

    fn closest_boundary_point(&self, ctx: &DrawContext<N, E, Ty, Ix>, pos: Pos2) -> Pos2 {
        let circle_radius = ctx.meta.canvas_to_screen_size(self.radius);
        let circle_center = ctx.meta.canvas_to_screen_pos(self.pos);

        let dir = pos - circle_center;
        circle_center + dir.normalized() * circle_radius
    }

    fn shape(&self, ctx: &DrawContext<N, E, Ty, Ix>) -> Vec<Shape> {
        let mut res = Vec::with_capacity(2);

        let is_interacted = self.selected || self.dragged;

        let style = match is_interacted {
            true => ctx.ctx.style().visuals.widgets.active,
            false => ctx.ctx.style().visuals.widgets.inactive,
        };
        let stroke = style.fg_stroke;

        let circle_center = ctx.meta.canvas_to_screen_pos(self.pos);
        let circle_radius = ctx.meta.canvas_to_screen_size(self.radius);
        let circle_shape = CircleShape {
            center: self.pos,
            radius: self.radius,
            fill: stroke.color,
            stroke,
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
                stroke.color,
            )
        });

        let label_shape = TextShape::new(label_pos, galley);
        res.push(label_shape.into());

        res
    }
}
