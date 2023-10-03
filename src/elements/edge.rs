use egui::{Color32, Context};

use super::StyleEdge;

/// Stores properties of an edge that can be changed. Used to apply changes to the graph.
#[derive(Clone, Debug)]
pub struct Edge<E: Clone> {
    /// Client data
    data: Option<E>,

    style: StyleEdge,
}

impl<E: Clone> Default for Edge<E> {
    fn default() -> Self {
        Self {
            style: Default::default(),

            data: Default::default(),
        }
    }
}

impl<E: Clone> Edge<E> {
    pub fn new(data: E) -> Self {
        Self {
            data: Some(data),

            ..Default::default()
        }
    }

    pub fn tip_angle(&self) -> f32 {
        self.style.tip_angle
    }

    pub fn data(&self) -> Option<&E> {
        self.data.as_ref()
    }

    pub fn color(&self, ctx: &Context) -> Color32 {
        ctx.style()
            .visuals
            .gray_out(ctx.style().visuals.widgets.inactive.fg_stroke.color)
    }

    pub fn width(&self) -> f32 {
        self.style.width
    }

    pub fn with_width(&mut self, width: f32) -> Self {
        let mut ne = self.clone();
        ne.style.width = width;
        ne
    }

    pub fn curve_size(&self) -> f32 {
        self.style.curve_size
    }

    pub fn tip_size(&self) -> f32 {
        self.style.tip_size
    }
}
