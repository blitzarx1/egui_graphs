use egui::{
    epaint::{CircleShape, TextShape},
    Context, FontFamily, FontId, Pos2, Stroke,
};
use petgraph::graph::IndexType;
use petgraph::EdgeType;

use crate::Node;

use super::{custom::DrawContext, Layers};

pub fn default_node_draw<N: Clone, E: Clone, Ty: EdgeType, Ix: IndexType>(
    ctx: &Context,
    n: &Node<N>,
    state: &DrawContext<N, E, Ty, Ix>,
    l: &mut Layers,
) {
    let is_interacted = n.selected() || n.dragged();
    let loc = n.screen_location(state.meta).to_pos2();
    let rad = match is_interacted {
        true => n.screen_radius(state.meta, state.style) * 1.5,
        false => n.screen_radius(state.meta, state.style),
    };

    let color = n.color(ctx);
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

    let show_label = state.style.labels_always || is_interacted;
    if !show_label {
        return;
    };

    let color = ctx.style().visuals.text_color();
    let label_pos = Pos2::new(loc.x, loc.y - rad * 2.);
    let label_size = rad;
    let galley = ctx.fonts(|f| {
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
