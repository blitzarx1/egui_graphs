use egui::{Color32, Vec2};

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

    /// This field is recomputed on every frame. Not available for client and not sent in changes.
    pub(crate) radius: f32,
    /// This field is recomputed on every frame. Not available for client and not sent in changes.
    pub(crate) selected_child: bool,
    /// This field is recomputed on every frame. Not available for client and not sent in changes.
    pub(crate) selected_parent: bool,
}

impl<N: Clone> Default for Node<N> {
    fn default() -> Self {
        Self {
            radius: 5.,

            location: Default::default(),
            data: Default::default(),
            color: Default::default(),
            selected: Default::default(),
            dragged: Default::default(),
            selected_child: Default::default(),
            selected_parent: Default::default(),
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

    pub fn screen_transform(&self, zoom: f32, pan: Vec2) -> Self {
        Self {
            location: self.location * zoom + pan,
            radius: self.radius * zoom,

            color: self.color,
            dragged: self.dragged,

            selected: self.selected,
            selected_child: self.selected_child,
            selected_parent: self.selected_parent,

            data: self.data.clone(),
        }
    }

    pub fn selected_child(&self) -> bool {
        self.selected_child
    }

    pub fn selected_parent(&self) -> bool {
        self.selected_parent
    }

    pub fn radius(&self) -> f32 {
        self.radius
    }

    pub fn selected(&self) -> bool {
        self.selected || self.selected_child || self.selected_parent
    }

    pub fn reset_precalculated(&mut self) {
        self.radius = 5.;
        self.selected_child = false;
        self.selected_parent = false;
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
    pub selected: bool,

    /// This field is recomputed on every frame. Not available for client and not sent in changes.
    pub(crate) selected_child: bool,
    /// This field is recomputed on every frame. Not available for client and not sent in changes.
    pub(crate) selected_parent: bool,
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
            selected: Default::default(),
            selected_child: Default::default(),
            selected_parent: Default::default(),
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

    pub fn screen_transform(&self, zoom: f32) -> Self {
        Self {
            width: self.width * zoom,
            tip_size: self.tip_size * zoom,
            curve_size: self.curve_size * zoom,

            color: self.color,
            tip_angle: self.tip_angle,
            selected: self.selected,

            selected_child: self.selected_child,
            selected_parent: self.selected_parent,

            data: self.data.clone(),
        }
    }

    pub fn selected_child(&self) -> bool {
        self.selected_child
    }

    pub fn selected_parent(&self) -> bool {
        self.selected_parent
    }

    pub fn selected(&self) -> bool {
        self.selected || self.selected_child || self.selected_parent
    }

    pub(crate) fn reset_precalculated(&mut self) {
        self.selected_child = false;
        self.selected_parent = false;
    }
}
