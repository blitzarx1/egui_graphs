use egui::{Painter, Shape};

/// Represents drawing layers of the frame. The bottom layer is drawn first, the top
/// layer is drawn last.
#[derive(Default)]
pub struct Layers {
    bottom: Vec<Shape>,
    top: Vec<Shape>,
}

impl Layers {
    pub fn add_bottom(&mut self, shape: impl Into<Shape>) {
        self.bottom.push(shape.into());
    }

    pub fn add_top(&mut self, shape: impl Into<Shape>) {
        self.top.push(shape.into());
    }

    pub fn draw(self, p: Painter) {
        self.bottom.into_iter().for_each(|shape| {
            p.add(shape);
        });
        self.top.into_iter().for_each(|shape| {
            p.add(shape);
        });
    }
}
