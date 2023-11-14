use egui::{
    epaint::{CircleShape, TextShape},
    FontFamily, FontId, Pos2, Stroke,
};
use petgraph::graph::IndexType;
use petgraph::EdgeType;

use crate::Node;

use super::{Layers, drawer::DrawContext};

pub fn default_node_draw<N: Clone, E: Clone, Ty: EdgeType, Ix: IndexType>(
    ctx: &DrawContext<N, E, Ty, Ix>,
    n: &Node<N, Ix>,
    l: &mut Layers,
) {
    let is_interacted = n.selected() || n.dragged();
    let loc = ctx.meta.canvas_to_screen_pos(n.location());
    let rad = match is_interacted {
        true => ctx.meta.canvas_to_screen_size(n.radius()) * 1.5,
        false => ctx.meta.canvas_to_screen_size(n.radius()),
    };

    let color = n.color(ctx.ctx);
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

    let show_label = ctx.style.labels_always || is_interacted;
    if !show_label {
        return;
    };

    let color = ctx.ctx.style().visuals.text_color();
    let label_pos = Pos2::new(loc.x, loc.y - rad * 2.);
    let label_size = rad;
    let galley = ctx.ctx.fonts(|f| {
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
