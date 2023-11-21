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
#[derive(Clone, Debug, Default)]
pub struct NodeProps {
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
    payload: N,

    props: NodeProps,
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
        let mut res = String::default();
        res += format!("id : {:?}", self.id).as_str();
        res += format!(", props: {:?}", self.props()).as_str();
        f.write_str(format!("Node {{{}}}", res).as_str())
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
            payload: self.payload().clone(),
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
        let props = NodeProps::default();
        let display = D::from(props.clone());
        Self {
            payload,
            props,
            display,

            id: Default::default(),
            _marker: Default::default(),
        }
    }

    pub fn props(&self) -> &NodeProps {
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
        &self.payload
    }

    pub fn payload_mut(&mut self) -> &mut N {
        &mut self.payload
    }

    pub fn location(&self) -> Pos2 {
        self.props.location
    }

    pub fn set_location(&mut self, loc: Pos2) {
        self.props.location = loc
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
}
