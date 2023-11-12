use egui::{Color32, Context, Pos2};

use crate::ComputedNode;

use super::StyleNode;

/// Stores properties of a node.
#[derive(Clone, Debug)]
pub struct Node<N: Clone> {
    payload: Option<N>,

    location: Pos2,

    label: String,

    style: StyleNode,

    selected: bool,
    dragged: bool,
    computed: ComputedNode,
}

impl<N: Clone> Node<N> {
    pub fn new(location: Pos2, data: N) -> Self {
        Self {
            location,
            payload: Some(data),
            style: Default::default(),
            label: Default::default(),
            selected: Default::default(),
            dragged: Default::default(),
            computed: Default::default(),
        }
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

    pub fn payload(&self) -> Option<&N> {
        self.payload.as_ref()
    }

    pub fn set_data(&mut self, data: Option<N>) {
        self.payload = data;
    }

    pub fn with_data(&self, data: Option<N>) -> Self {
        let mut res = self.clone();
        res.payload = data;
        res
    }

    pub fn location(&self) -> Pos2 {
        self.location
    }

    pub fn set_location(&mut self, loc: Pos2) {
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
