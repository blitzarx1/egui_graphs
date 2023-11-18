use egui::Pos2;
use petgraph::stable_graph::{DefaultIx, IndexType, NodeIndex};

/// Stores properties of a node.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug)]
pub struct Node<N: Clone, Ix: IndexType = DefaultIx> {
    id: Option<NodeIndex<Ix>>,
    location: Option<Pos2>,

    payload: N,
    label: String,

    selected: bool,
    dragged: bool,
}

impl<N: Clone, Ix: IndexType> Node<N, Ix> {
    pub fn new(payload: N) -> Self {
        Self {
            payload,

            id: Default::default(),
            location: Default::default(),
            label: Default::default(),
            selected: Default::default(),
            dragged: Default::default(),
        }
    }

    /// Binds node to the actual node and position in the graph.
    pub fn bind(&mut self, id: NodeIndex<Ix>, location: Pos2) {
        self.id = Some(id);
        self.location = Some(location);
    }

    // TODO: handle unbinded node
    pub fn id(&self) -> NodeIndex<Ix> {
        self.id.unwrap()
    }

    pub fn payload(&self) -> &N {
        &self.payload
    }

    pub fn payload_mut(&mut self) -> &mut N {
        &mut self.payload
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
