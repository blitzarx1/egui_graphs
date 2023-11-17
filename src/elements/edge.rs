use egui::{Color32, Context};
use petgraph::stable_graph::{DefaultIx, EdgeIndex, IndexType};

/// Uniquely identifies edge with source, target and index in the set of duplicate edges.
#[cfg_attr(feature="serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug)]
pub struct EdgeID<Ix: IndexType = DefaultIx> {
    pub idx: EdgeIndex<Ix>,

    /// Index of the edge among siblings.
    pub order: usize,
}

impl<Ix: IndexType> EdgeID<Ix> {
    pub fn new(idx: EdgeIndex<Ix>) -> Self {
        Self {
            idx,
            order: Default::default(),
        }
    }

    pub fn with_order(mut self, order: usize) -> Self {
        self.order = order;
        self
    }
}

/// Stores properties of an edge that can be changed. Used to apply changes to the graph.
#[cfg_attr(feature="serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug)]
pub struct Edge<E: Clone, Ix: IndexType = DefaultIx> {
    id: Option<EdgeID<Ix>>,

    /// Client data
    pub data: E,

    selected: bool,
}

impl<E: Clone, Ix: IndexType> Edge<E, Ix> {
    pub fn new(data: E) -> Self {
        Self {
            data,

            id: Default::default(),
            selected: Default::default(),
        }
    }

    /// Binds node to the actual node and position in the graph.
    pub fn bind(&mut self, idx: EdgeIndex<Ix>, order: usize) {
        let id = EdgeID::new(idx).with_order(order);
        self.id = Some(id);
    }

    pub fn id(&self) -> EdgeID<Ix> {
        self.id.clone().unwrap()
    }

    // TODO: handle unwrap
    pub fn order(&self) -> usize {
        self.id.as_ref().unwrap().order
    }

    // TODO: handle unwrap
    pub fn set_order(&mut self, order: usize) {
        self.id.as_mut().unwrap().order = order;
    }

    pub fn color(&self, ctx: &Context) -> Color32 {
        if self.selected {
            return ctx.style().visuals.widgets.hovered.fg_stroke.color;
        }

        ctx.style()
            .visuals
            .gray_out(ctx.style().visuals.widgets.inactive.fg_stroke.color)
    }

    pub fn set_selected(&mut self, selected: bool) {
        self.selected = selected;
    }

    pub fn selected(&self) -> bool {
        self.selected
    }
}
