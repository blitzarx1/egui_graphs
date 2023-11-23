use std::fmt::Debug;
use std::marker::PhantomData;

use egui::Pos2;
use petgraph::{
    stable_graph::{DefaultIx, IndexType, NodeIndex},
    EdgeType,
};

use crate::{DefaultNodeShape, DisplayNode};

/// Stores properties of a [Node]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug)]
pub struct NodeProps<N: Clone> {
    pub payload: N,
    pub location: Pos2,
    pub label: String,
    pub selected: bool,
    pub dragged: bool,
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Node<N, E, Ty, Ix = DefaultIx, D = DefaultNodeShape>
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
    pub fn new(payload: N) -> Self {
        let props = NodeProps {
            payload,
            location: Pos2::default(),
            label: String::default(),
            selected: bool::default(),
            dragged: bool::default(),
        };

        let display = D::from(props.clone());
        Self {
            props,
            display,

            id: Default::default(),
            _marker: Default::default(),
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

    /// TODO: rethink this
    /// Binds node to the actual node and position in the graph.
    pub fn bind(&mut self, id: NodeIndex<Ix>, location: Pos2) {
        self.id = Some(id);
        self.props.location = location;
    }

    // TODO: handle unbinded node
    pub fn id(&self) -> NodeIndex<Ix> {
        self.id.unwrap()
    }

    pub fn payload(&self) -> &N {
        &self.props.payload
    }

    pub fn payload_mut(&mut self) -> &mut N {
        &mut self.props.payload
    }

    pub fn location(&self) -> Pos2 {
        self.props.location
    }

    pub fn set_location(&mut self, loc: Pos2) {
        self.props.location = loc
    }

    pub fn with_location(mut self, loc: Pos2) -> Self {
        self.props.location = loc;
        self
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
        self.props.label = label
    }

    pub fn with_label(mut self, label: String) -> Self {
        self.props.label = label;
        self
    }
}
