use std::marker::PhantomData;

use petgraph::{
    stable_graph::{DefaultIx, EdgeIndex, IndexType},
    Directed, EdgeType,
};
use serde::{Deserialize, Serialize};

use crate::{DefaultEdgeShape, DefaultNodeShape, DisplayEdge, DisplayNode};

/// Stores properties of an [Edge]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EdgeProps<E: Clone> {
    pub payload: E,
    pub order: usize,
    pub selected: bool,
    pub label: String,
}

/// Stores properties of an edge that can be changed. Used to apply changes to the graph.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Edge<
    N: Clone,
    E: Clone,
    Ty: EdgeType = Directed,
    Ix: IndexType = DefaultIx,
    Dn: DisplayNode<N, E, Ty, Ix> = DefaultNodeShape,
    D: DisplayEdge<N, E, Ty, Ix, Dn> = DefaultEdgeShape,
> {
    id: Option<EdgeIndex<Ix>>,

    display: D,

    props: EdgeProps<E>,
    _marker: PhantomData<(N, Ty, Dn)>,
}

impl<
        N: Clone,
        E: Clone,
        Ty: EdgeType,
        Ix: IndexType,
        Dn: DisplayNode<N, E, Ty, Ix>,
        D: DisplayEdge<N, E, Ty, Ix, Dn>,
    > Edge<N, E, Ty, Ix, Dn, D>
{
    pub fn new(payload: E) -> Self {
        let props = EdgeProps {
            payload,

            order: usize::default(),
            selected: bool::default(),
            label: String::default(),
        };

        let display = D::from(props.clone());
        Self {
            props,
            display,

            id: Option::default(),
            _marker: PhantomData,
        }
    }

    pub fn props(&self) -> &EdgeProps<E> {
        &self.props
    }

    pub fn display(&self) -> &D {
        &self.display
    }

    pub fn display_mut(&mut self) -> &mut D {
        &mut self.display
    }

    #[allow(clippy::missing_panics_doc)] // TODO: Add panic message
    pub fn id(&self) -> EdgeIndex<Ix> {
        self.id.unwrap()
    }

    pub(crate) fn set_id(&mut self, id: EdgeIndex<Ix>) {
        self.id = Some(id);
    }

    pub fn order(&self) -> usize {
        self.props.order
    }

    pub(crate) fn set_order(&mut self, order: usize) {
        self.props.order = order;
    }

    pub fn payload(&self) -> &E {
        &self.props.payload
    }

    pub fn payload_mut(&mut self) -> &mut E {
        &mut self.props.payload
    }

    pub fn set_selected(&mut self, selected: bool) {
        self.props.selected = selected;
    }

    pub fn selected(&self) -> bool {
        self.props.selected
    }

    pub fn set_label(&mut self, label: String) {
        self.props.label = label;
    }

    pub fn label(&self) -> String {
        self.props.label.clone()
    }
}
