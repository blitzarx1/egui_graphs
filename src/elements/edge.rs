use egui::Color32;

use crate::state_computed::StateComputedEdge;

use super::StyleEdge;

/// Stores properties of an edge that can be changed. Used to apply changes to the graph.
#[derive(Clone, Debug)]
pub struct Edge<E: Clone> {
    /// Client data
    data: Option<E>,

    selected_child: bool,
    selected_parent: bool,

    style: StyleEdge,
}

impl<E: Clone> Default for Edge<E> {
    fn default() -> Self {
        Self {
            selected_child: Default::default(),
            selected_parent: Default::default(),
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

    pub fn color(&self) -> Color32 {
        if self.selected_child {
            return self.style.color.interaction.selection_child;
        }

        if self.selected_parent {
            return self.style.color.interaction.selection_parent;
        }

        self.style.color.main
    }

    pub fn with_color(&mut self, color: Color32) -> Self {
        let mut ne = self.clone();
        ne.style.color.main = color;
        ne
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

    pub fn subselected(&self) -> bool {
        self.selected_child || self.selected_parent
    }

    pub fn selected_child(&self) -> bool {
        self.selected_child
    }

    pub fn selected_parent(&self) -> bool {
        self.selected_parent
    }

    pub fn apply_computed_props(&mut self, comp: &StateComputedEdge) {
        self.selected_child = comp.selected_child;
        self.selected_parent = comp.selected_parent;
    }
}
