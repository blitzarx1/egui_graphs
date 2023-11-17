use egui::Pos2;
use petgraph::stable_graph::{IndexType, NodeIndex};

use crate::ComputedNode;

/// Stores properties of a node.
#[derive(Clone, Debug)]
pub struct Node<N: Clone, Ix: IndexType> {
    id: Option<NodeIndex<Ix>>,
    location: Option<Pos2>,

    payload: Option<N>,
    label: String,

    selected: bool,
    dragged: bool,
    computed: ComputedNode,
}

impl<N: Clone, Ix: IndexType> Node<N, Ix> {
    pub fn new(payload: N) -> Self {
        Self {
            payload: Some(payload),

            id: Default::default(),
            location: Default::default(),
            label: Default::default(),
            selected: Default::default(),
            dragged: Default::default(),
            computed: Default::default(),
        }
    }

    /// Binds node to the actual node and position in the graph.
    pub fn bind(&mut self, id: NodeIndex<Ix>, location: Pos2) {
        self.id = Some(id);
        self.location = Some(location);
    }

    pub fn id(&self) -> NodeIndex<Ix> {
        self.id.unwrap()
    }

    pub fn num_connections(&self) -> usize {
        self.computed.num_connections
    }

    pub(crate) fn set_computed(&mut self, comp: ComputedNode) {
        self.computed = comp;
    }

    pub fn payload(&self) -> Option<&N> {
        self.payload.as_ref()
    }

    pub fn payload_mut(&mut self) -> Option<&mut N> {
        self.payload.as_mut()
    }

    pub fn set_data(&mut self, data: Option<N>) {
        self.payload = data;
    }

    pub fn with_data(&self, data: Option<N>) -> Self {
        let mut res = self.clone();
        res.payload = data;
        res
    }

    // TODO: handle unbinded node
    pub fn location(&self) -> Pos2 {
        self.location.unwrap()
    }

    pub fn set_location(&mut self, loc: Pos2) {
        self.location = Some(loc)
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
}
