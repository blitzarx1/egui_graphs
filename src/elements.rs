use egui::{Color32, Vec2};

use crate::metadata::Metadata;

/// Stores properties of a node that can be changed. Used to apply changes to the graph.
#[derive(Clone, Debug, PartialEq)]
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
}

impl<N: Clone> Default for Node<N> {
    fn default() -> Self {
        Self {
            location: Default::default(),
            data: Default::default(),
            label: Default::default(),
            color: Default::default(),
            folded: Default::default(),
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

    pub fn data(&self) -> Option<&N> {
        self.data.as_ref()
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

    pub fn screen_transform(&self, meta: &Metadata) -> Self {
        Self {
            location: self.location * meta.zoom + meta.pan,

            color: self.color,
            dragged: self.dragged,

            label: self.label.clone(),
            folded: self.folded,
            selected: self.selected,
            data: self.data.clone(),
        }
    }
}

/// Stores properties of an edge that can be changed. Used to apply changes to the graph.
#[derive(Clone, Debug, Copy, PartialEq)]
pub struct Edge<E: Clone> {
    /// Client data
    data: Option<E>,

    width: f32,
    tip_size: f32,
    tip_angle: f32,
    curve_size: f32,

    /// If `color` is None default color is used.
    color: Option<Color32>,
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

    pub fn width(&self) -> f32 {
        self.width
    }

    pub fn tip_size(&self) -> f32 {
        self.tip_size
    }

    pub fn tip_angle(&self) -> f32 {
        self.tip_angle
    }

    pub fn curve_size(&self) -> f32 {
        self.curve_size
    }

    pub fn data(&self) -> Option<&E> {
        self.data.as_ref()
    }

    pub fn color(&self) -> Option<Color32> {
        self.color
    }

    pub fn with_color(&mut self, color: Color32) -> Self {
        let mut res = self.clone();
        res.color = Some(color);
        res
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn node_default() {
        let node: Node<()> = Node::default();
        assert_eq!(node.location, Vec2::default());
        assert_eq!(node.data, None);
        assert_eq!(node.color, None);
        assert!(!node.selected);
        assert!(!node.dragged);
    }

    #[test]
    fn node_new() {
        let node = Node::new(Vec2::new(1., 2.), "data");
        assert_eq!(node.location, Vec2::new(1., 2.));
        assert_eq!(node.data, Some("data"));
        assert_eq!(node.color, None);
        assert!(!node.selected);
        assert!(!node.dragged);
    }

    #[test]
    fn node_screen_transform() {
        let mut node = Node::new(Vec2::new(1., 2.), "data");
        let meta = Metadata {
            zoom: 2.,
            pan: Vec2::new(3., 4.),
            ..Default::default()
        };

        node = node.screen_transform(&meta);
        assert_eq!(node.location, Vec2::new(5., 8.));
        assert_eq!(node.data, Some("data"));
        assert_eq!(node.color, None);
        assert!(!node.selected);
        assert!(!node.dragged);
    }

    #[test]
    fn edge_default() {
        let edge: Edge<()> = Edge::default();
        assert_eq!(edge.width, 2.);
        assert_eq!(edge.tip_size, 15.);
        assert_eq!(edge.tip_angle, std::f32::consts::TAU / 50.);
        assert_eq!(edge.curve_size, 20.);
        assert_eq!(edge.data, None);
        assert_eq!(edge.color, None);
    }

    #[test]
    fn edge_new() {
        let edge = Edge::new("data");
        assert_eq!(edge.width, 2.);
        assert_eq!(edge.tip_size, 15.);
        assert_eq!(edge.tip_angle, std::f32::consts::TAU / 50.);
        assert_eq!(edge.curve_size, 20.);
        assert_eq!(edge.data, Some("data"));
        assert_eq!(edge.color, None);
    }

    #[test]
    fn edge_screen_transform() {
        let mut edge = Edge::new("data");
        let meta = Metadata {
            zoom: 2.,
            pan: Vec2::new(3., 4.),
            ..Default::default()
        };

        edge = edge.screen_transform(&meta);
        assert_eq!(edge.width, 4.);
        assert_eq!(edge.tip_size, 30.);
        assert_eq!(edge.tip_angle, std::f32::consts::TAU / 50.);
        assert_eq!(edge.curve_size, 40.);
        assert_eq!(edge.data, Some("data"));
        assert_eq!(edge.color, None);
    }
}
