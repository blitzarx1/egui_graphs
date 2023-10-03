use egui::{Painter, Shape};

/// Represents drawing layers of the frame. The bottom layer is drawn first, the top
/// layer is drawn last.
#[derive(Default)]
pub struct Layers {
    bottom: Vec<Shape>,
    top: Vec<Shape>,
}

impl Layers {
    /// Default drawing layer. This is the bottom layer.
    pub fn add(&mut self, shape: impl Into<Shape>) {
        self.bottom.push(shape.into());
    }

    /// Drawing layer for prioritizing shapes drawings. This is the top layer.
    pub fn add_top(&mut self, shape: impl Into<Shape>) {
        self.top.push(shape.into());
    }

    pub(crate) fn draw(self, p: Painter) {
        self.bottom.into_iter().for_each(|shape| {
            p.add(shape);
        });
        self.top.into_iter().for_each(|shape| {
            p.add(shape);
        });
    }
}
