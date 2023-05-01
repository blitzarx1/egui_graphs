use egui::{Color32, Vec2};

/// Stores properties of a node that can be changed. Used to apply changes to the graph.
#[derive(Clone, Debug, Copy, PartialEq)]
pub struct Node<N: Clone> {
    /// Client data
    pub data: N,

    pub location: Vec2,

    /// If `color` is None default color is used.
    pub color: Option<Color32>,
    pub(crate) radius: f32,

    pub selected: bool,
    pub dragged: bool,
}

impl<N: Clone> Node<N> {
    pub fn new(location: Vec2, data: N) -> Self {
        Self {
            location,
            data,

            color: None,
            radius: 5.,

            selected: false,
            dragged: false,
        }
    }

    pub fn screen_transform(&self, zoom: f32, pan: Vec2) -> Self {
        Self {
            location: self.location * zoom + pan,
            radius: self.radius * zoom,

            color: self.color,
            selected: self.selected,
            dragged: self.dragged,

            data: self.data.clone(),
        }
    }

    pub fn radius(&self) -> f32 {
        self.radius
    }
}

/// Stores properties of an edge that can be changed. Used to apply changes to the graph.
#[derive(Clone, Debug, Copy, PartialEq)]
pub struct Edge<E: Clone> {
    /// Client data
    pub data: E,

    pub width: f32,
    pub tip_size: f32,
    pub tip_angle: f32,
    pub curve_size: f32,

    /// If `color` is None default color is used.
    pub color: Option<Color32>,
    pub selected: bool,
}

impl<E: Clone> Edge<E> {
    pub fn new(data: E) -> Self {
        Self {
            data,

            width: 2.,
            tip_size: 15.,
            tip_angle: std::f32::consts::TAU / 50.,
            curve_size: 20.,

            color: None,
            selected: false,
        }
    }

    pub fn screen_transform(&self, zoom: f32) -> Self {
        Self {
            width: self.width * zoom,
            tip_size: self.tip_size * zoom,
            curve_size: self.curve_size * zoom,

            color: self.color,
            tip_angle: self.tip_angle,
            selected: self.selected,

            data: self.data.clone(),
        }
    }
}
