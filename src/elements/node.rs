use egui::{Color32, Context, Vec2};

use crate::{metadata::Metadata, ComputedNode};

use super::StyleNode;

/// Stores properties of a node.
#[derive(Clone, Debug)]
pub struct Node<N: Clone> {
    /// Client data
    data: Option<N>,

    location: Vec2,

    label: String,

    style: StyleNode,

    selected: bool,
    dragged: bool,
    computed: ComputedNode,
}

impl<N: Clone> Node<N> {
    pub fn new(location: Vec2, data: N) -> Self {
        Self {
            location,
            data: Some(data),
            style: Default::default(),
            label: Default::default(),
            selected: Default::default(),
            dragged: Default::default(),
            computed: Default::default(),
        }
    }

    /// Returns actual location of the node on the screen. It accounts for the current zoom and pan values.
    pub fn screen_location(&self, m: &Metadata) -> Vec2 {
        self.location * m.zoom + m.pan
    }

    /// Returns actual radius of the node on the screen. It accounts for the number of connections and current zoom value.
    pub fn screen_radius(&self, m: &Metadata) -> f32 {
        (self.radius() + self.num_connections() as f32) * m.zoom
    }

    pub fn radius(&self) -> f32 {
        self.style.radius
    }

    pub fn num_connections(&self) -> usize {
        self.computed.num_connections
    }

    pub fn set_radius(&mut self, new_rad: f32) {
        self.style.radius = new_rad
    }

    pub(crate) fn set_computed(&mut self, comp: ComputedNode) {
        self.computed = comp;
    }

    pub fn data(&self) -> Option<&N> {
        self.data.as_ref()
    }

    pub fn set_data(&mut self, data: Option<N>) {
        self.data = data;
    }

    pub fn with_data(&self, data: Option<N>) -> Self {
        let mut res = self.clone();
        res.data = data;
        res
    }

    pub fn location(&self) -> Vec2 {
        self.location
    }

    pub fn set_location(&mut self, loc: Vec2) {
        self.location = loc
    }

    pub fn selected(&self) -> bool {
        self.selected
    }

    pub fn set_selected(&mut self, selected: bool) {
        self.selected = selected;
    }

    pub fn dragged(&self) -> bool {
        self.dragged
    }

    pub fn set_dragged(&mut self, dragged: bool) {
        self.dragged = dragged;
    }

    pub fn label(&self) -> String {
        self.label.clone()
    }

    pub fn with_label(&mut self, label: String) -> Self {
        let mut res = self.clone();
        res.label = label;
        res
    }

    pub fn color(&self, ctx: &Context) -> Color32 {
        if self.dragged {
            return ctx.style().visuals.widgets.active.fg_stroke.color;
        }

        if self.selected {
            return ctx.style().visuals.widgets.hovered.fg_stroke.color;
        }

        ctx.style().visuals.widgets.inactive.fg_stroke.color
    }
}
