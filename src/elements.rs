use egui::{Color32, Vec2};

use crate::{
    metadata::Metadata,
    state_computed::{StateComputedEdge, StateComputedNode},
};

/// Stores properties of a node that can be changed. Used to apply changes to the graph.
#[derive(Clone, Debug, Copy, PartialEq)]
pub struct Node<N: Clone> {
    /// Client data
    pub data: Option<N>,

    pub location: Vec2,

    /// If `color` is None default color is used.
    pub color: Option<Color32>,

    pub selected: bool,
    pub dragged: bool,
}

impl<N: Clone> Default for Node<N> {
    fn default() -> Self {
        Self {
            location: Default::default(),
            data: Default::default(),
            color: Default::default(),
            selected: Default::default(),
            dragged: Default::default(),
        }
    }
}

impl<N: Clone> Node<N> {
    pub fn new(location: Vec2, data: N) -> Self {
        Self {
            location,
            data: Some(data),

            ..Default::default()
        }
    }

    pub fn screen_transform(&self, meta: &Metadata) -> Self {
        Self {
            location: self.location * meta.zoom + meta.pan,

            color: self.color,
            dragged: self.dragged,

            selected: self.selected,
            data: self.data.clone(),
        }
    }
}

/// Stores properties of an edge that can be changed. Used to apply changes to the graph.
#[derive(Clone, Debug, Copy, PartialEq)]
pub struct Edge<E: Clone> {
    /// Client data
    pub data: Option<E>,

    pub width: f32,
    pub tip_size: f32,
    pub tip_angle: f32,
    pub curve_size: f32,

    /// If `color` is None default color is used.
    pub color: Option<Color32>,
}

impl<E: Clone> Default for Edge<E> {
    fn default() -> Self {
        Self {
            width: 2.,
            tip_size: 15.,
            tip_angle: std::f32::consts::TAU / 50.,
            curve_size: 20.,

            data: Default::default(),
            color: Default::default(),
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

    pub(crate) fn screen_transform(&self, meta: &Metadata) -> Self {
        Self {
            width: self.width * meta.zoom,
            tip_size: self.tip_size * meta.zoom,
            curve_size: self.curve_size * meta.zoom,

            color: self.color,
            tip_angle: self.tip_angle,

            data: self.data.clone(),
        }
    }
}
