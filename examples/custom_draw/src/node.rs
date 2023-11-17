use egui::{epaint::TextShape, FontFamily, FontId, Pos2, Rect, Rounding, Shape, Stroke, Vec2};
use egui_graphs::{DisplayNode, Graph, Interactable, Node};
use petgraph::{stable_graph::IndexType, EdgeType};

pub struct NodeShape {
    label: String,
    loc: Pos2,
    selected: bool,
    dragged: bool,

    size: f32,
}

impl<N: Clone, Ix: IndexType> From<Node<N, Ix>> for NodeShape {
    fn from(node: Node<N, Ix>) -> Self {
        Self {
            label: node.label(),
            loc: node.location(),
            selected: node.selected(),
            dragged: node.dragged(),

            size: 30.,
        }
    }
}

impl<N: Clone, E: Clone, Ty: EdgeType, Ix: IndexType> Interactable<N, E, Ty, Ix> for NodeShape {
    fn is_inside<Dn: egui_graphs::DisplayNode<N, E, Ty, Ix>>(
        &self,
        _g: &Graph<N, E, Ty, Ix>,
        pos: Pos2,
    ) -> bool {
        let rect = Rect::from_center_size(self.loc, Vec2::new(self.size, self.size));

        rect.contains(pos)
    }
}

impl<N: Clone, E: Clone, Ty: EdgeType, Ix: IndexType> DisplayNode<N, E, Ty, Ix> for NodeShape {
    fn closest_boundary_point(&self, dir: Vec2) -> Pos2 {
        let margin = 5.0;
        find_intersection(self.loc, self.size + margin, dir)
    }

    fn shapes(&mut self, ctx: &egui_graphs::DrawContext<N, E, Ty, Ix>) -> Vec<egui::Shape> {
        // lets draw a rect with label in the center for every node

        // find node center location on the screen coordinates
        let center = ctx.meta.canvas_to_screen_pos(self.loc);
        let size = ctx.meta.canvas_to_screen_size(self.size);
        let rect = Rect::from_center_size(center, Vec2::new(size, size));

        let interacted = self.selected || self.dragged;
        let rect_color = match interacted {
            true => ctx.ctx.style().visuals.selection.bg_fill,
            false => ctx.ctx.style().visuals.weak_text_color(),
        };

        let shape_rect = Shape::rect_stroke(rect, Rounding::default(), Stroke::new(1., rect_color));

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
        let shape_label = TextShape::new(rect.center() + offset, galley);

        vec![shape_rect, shape_label.into()]
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
