use egui::{Color32, Vec2};

use crate::{metadata::Metadata, StateComputedNode};

use super::StyleNode;

/// Stores properties of a node.
#[derive(Clone, Debug)]
pub struct Node<N: Clone> {
    /// Client data
    data: Option<N>,

    location: Vec2,

    label: Option<String>,

    style: StyleNode,

    folded: bool,
    selected: bool,
    dragged: bool,

    subselected_child: bool,
    subselected_parent: bool,
    subfolded: bool,
    num_folded: usize,
    num_connections: usize,
}

impl<N: Clone> Node<N> {
    pub fn new(location: Vec2, data: N) -> Self {
        Self {
            location,
            data: Some(data),
            style: Default::default(),
            subfolded: Default::default(),
            subselected_child: Default::default(),
            subselected_parent: Default::default(),
            label: Default::default(),
            folded: Default::default(),
            selected: Default::default(),
            num_folded: Default::default(),
            dragged: Default::default(),
            num_connections: Default::default(),
        }
    }

    pub fn screen_location(&self, m: &Metadata) -> Vec2 {
        self.location * m.zoom + m.pan
    }

    pub fn screen_radius(&self, m: &Metadata) -> f32 {
        self.style.radius * m.zoom
    }

    pub fn visible(&self) -> bool {
        !self.subfolded
    }

    pub fn subselected(&self) -> bool {
        self.subselected_child || self.subselected_parent
    }

    pub fn radius(&self) -> f32 {
        self.style.radius
    }

    pub fn num_connections(&self) -> usize {
        self.num_connections
    }

    pub fn num_folded(&self) -> usize {
        self.num_folded
    }

    pub fn set_radius(&mut self, new_rad: f32) {
        self.style.radius = new_rad
    }

    pub fn subfolded(&self) -> bool {
        self.subfolded
    }

    pub fn subselected_child(&self) -> bool {
        self.subselected_child
    }

    pub fn subselected_parent(&self) -> bool {
        self.subselected_parent
    }

    pub(crate) fn apply_computed_props(&mut self, comp: StateComputedNode) {
        self.subfolded = comp.subfolded();
        self.subselected_child = comp.selected_child;
        self.subselected_parent = comp.selected_parent;
        self.num_folded = comp.num_folded;
        self.num_connections = comp.num_connections;
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

    pub fn folded(&self) -> bool {
        self.folded
    }

    pub fn set_folded(&mut self, folded: bool) {
        self.folded = folded;
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

    pub fn label(&self) -> Option<&String> {
        self.label.as_ref()
    }

    pub fn with_label(&mut self, label: String) -> Self {
        let mut res = self.clone();
        res.label = Some(label);
        res
    }

    // TODO: use ctx.style().visuals for color selection
    pub fn color(&self) -> Color32 {
        if self.folded() {
            return Color32::TRANSPARENT;
        }

        if self.dragged() {
            return self.style.color.interaction.drag;
        }

        if self.selected() {
            return self.style.color.interaction.selection;
        }

        if self.subselected_child() {
            return self.style.color.interaction.selection_child;
        }

        if self.subselected_parent() {
            return self.style.color.interaction.selection_parent;
        }

        self.style.color.main
    }

    pub fn with_color(&mut self, color: Color32) -> Self {
        let mut nn = self.clone();
        nn.style.color.main = color;
        nn
    }
}
