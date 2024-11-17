use std::fmt::Debug;
use std::marker::PhantomData;

use egui::{Color32, Pos2};
use petgraph::{
    stable_graph::{DefaultIx, IndexType, NodeIndex},
    Directed, EdgeType,
};
use serde::{Deserialize, Serialize};

use crate::{DefaultNodeShape, DisplayNode};

/// Stores properties of a [Node]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NodeProps<N>
where
    N: Clone,
{
    pub payload: N,
    pub label: String,
    pub selected: bool,
    pub dragged: bool,
    pub location: Pos2,

    color: Option<Color32>,
}

impl<N> NodeProps<N>
where
    N: Clone,
{
    pub fn color(&self) -> Option<Color32> {
        self.color
    }
}

#[derive(Serialize, Deserialize)]
pub struct Node<N, E, Ty = Directed, Ix = DefaultIx, D = DefaultNodeShape>
where
    N: Clone,
    E: Clone,
    Ty: EdgeType,
    Ix: IndexType,
    D: DisplayNode<N, E, Ty, Ix>,
{
    id: Option<NodeIndex<Ix>>,

    props: NodeProps<N>,
    display: D,

    _marker: PhantomData<(E, Ty)>,
}

#[allow(clippy::missing_fields_in_debug)] // TODO: add all fields or remove this and fix all warnings
impl<N, E, Ty, Ix, D> Debug for Node<N, E, Ty, Ix, D>
where
    N: Clone,
    E: Clone,
    Ty: EdgeType,
    Ix: IndexType,
    D: DisplayNode<N, E, Ty, Ix>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Node").field("id", &self.id).finish()
    }
}

impl<N, E, Ty, Ix, D> Clone for Node<N, E, Ty, Ix, D>
where
    N: Clone,
    E: Clone,
    Ty: EdgeType,
    Ix: IndexType,
    D: DisplayNode<N, E, Ty, Ix>,
{
    fn clone(&self) -> Self {
        let idx = self.id().index();
        Self {
            id: Some(NodeIndex::new(idx)),
            props: self.props.clone(),
            display: self.display.clone(),
            _marker: PhantomData,
        }
    }
}

impl<N, E, Ty, Ix, D> Node<N, E, Ty, Ix, D>
where
    N: Clone,
    E: Clone,
    Ty: EdgeType,
    Ix: IndexType,
    D: DisplayNode<N, E, Ty, Ix>,
{
    /// Creates a new node with default properties
    pub fn new(payload: N) -> Self {
        let props = NodeProps {
            payload,
            location: Pos2::default(),
            color: Option::default(),
            label: String::default(),
            selected: bool::default(),
            dragged: bool::default(),
        };

        Node::new_with_props(props)
    }

    /// Creates a new node with custom properties
    pub fn new_with_props(props: NodeProps<N>) -> Self {
        let display = D::from(props.clone());
        Self {
            props,
            display,

            id: Option::default(),
            _marker: PhantomData,
        }
    }

    pub fn props(&self) -> &NodeProps<N> {
        &self.props
    }

    pub fn display(&self) -> &D {
        &self.display
    }

    pub fn display_mut(&mut self) -> &mut D {
        &mut self.display
    }

    #[allow(clippy::missing_panics_doc)] // TODO: Add panic message
    pub fn id(&self) -> NodeIndex<Ix> {
        self.id.unwrap()
    }

    pub(crate) fn set_id(&mut self, id: NodeIndex<Ix>) {
        self.id = Some(id);
    }

    pub fn payload(&self) -> &N {
        &self.props.payload
    }

    pub fn payload_mut(&mut self) -> &mut N {
        &mut self.props.payload
    }

    pub fn color(&self) -> Option<Color32> {
        self.props.color()
    }

    pub fn set_color(&mut self, color: Color32) {
        self.props.color = Some(color);
    }

    pub fn location(&self) -> Pos2 {
        self.props.location
    }

    pub fn set_location(&mut self, loc: Pos2) {
        self.props.location = loc;
    }

    pub fn selected(&self) -> bool {
        self.props.selected
    }

    pub fn set_selected(&mut self, selected: bool) {
        self.props.selected = selected;
    }

    pub fn dragged(&self) -> bool {
        self.props.dragged
    }

    pub fn set_dragged(&mut self, dragged: bool) {
        self.props.dragged = dragged;
    }

    pub fn label(&self) -> String {
        self.props.label.clone()
    }

    pub fn set_label(&mut self, label: String) {
        self.props.label = label;
    }

    pub fn with_label(mut self, label: String) -> Self {
        self.props.label = label;
        self
    }
}
