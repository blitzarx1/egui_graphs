use egui::{Color32, Vec2};

use crate::{metadata::Metadata, StateComputedNode};

/// Stores properties of a node.
#[derive(Clone, Debug)]
pub struct Node<N: Clone> {
    /// Client data
    data: Option<N>,

    location: Vec2,

    label: Option<String>,

    /// If `color` is None default color is used.
    color: Option<Color32>,

    folded: bool,
    selected: bool,
    dragged: bool,

    subselected_child: bool,
    subselected_parent: bool,
    subfolded: bool,
    folded_num: usize,
    radius: f32,
}

impl<N: Clone> Default for Node<N> {
    fn default() -> Self {
        Self {
            radius: 5.,
            subfolded: Default::default(),
            subselected_child: Default::default(),
            subselected_parent: Default::default(),
            location: Default::default(),
            data: Default::default(),
            label: Default::default(),
            color: Default::default(),
            folded: Default::default(),
            selected: Default::default(),
            folded_num: Default::default(),
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

    pub fn screen_location(&self, m: &Metadata) -> Vec2 {
        self.location * m.zoom + m.pan
    }

    pub fn screen_radius(&self, m: &Metadata) -> f32 {
        self.radius * m.zoom
    }

    pub fn visible(&self) -> bool {
        !self.subfolded
    }

    pub fn subselected(&self) -> bool {
        self.subselected_child || self.subselected_parent
    }

    pub fn radius(&self) -> f32 {
        self.radius
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
        self.location = comp.location;
        self.radius = comp.radius;
        self.subfolded = comp.subfolded();
        self.subselected_child = comp.selected_child;
        self.subselected_parent = comp.selected_parent;
        self.folded_num = comp.num_folded;
    }

    pub fn folded_num(&self) -> usize {
        self.folded_num
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

    pub fn color(&self) -> Option<Color32> {
        self.color
    }

    pub fn with_color(&mut self, color: Color32) -> Self {
        let mut res = self.clone();
        res.color = Some(color);
        res
    }
}
