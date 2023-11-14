use egui::{Color32, Context};
use petgraph::stable_graph::{EdgeIndex, IndexType};

/// Uniquely identifies edge with source, target and index in the set of duplicate edges.
#[derive(Clone, Debug)]
pub struct EdgeID<Ix: IndexType> {
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

// TODO: move to shape
#[derive(Clone, Debug)]
pub struct StyleEdge {
    pub width: f32,
    pub tip_size: f32,
    pub tip_angle: f32,
    pub curve_size: f32,
    pub loop_size: f32,
}

impl Default for StyleEdge {
    fn default() -> Self {
        Self {
            width: 2.,
            tip_size: 15.,
            tip_angle: std::f32::consts::TAU / 30.,
            curve_size: 20.,
            loop_size: 3.,
        }
    }
}

/// Stores properties of an edge that can be changed. Used to apply changes to the graph.
#[derive(Clone, Debug)]
pub struct Edge<E: Clone, Ix: IndexType> {
    id: Option<EdgeID<Ix>>,

    /// Client data
    payload: Option<E>,

    style: StyleEdge,

    selected: bool,
}

impl<E: Clone, Ix: IndexType> Edge<E, Ix> {
    pub fn new(payload: E) -> Self {
        Self {
            payload: Some(payload),

            id: Default::default(),
            style: Default::default(),
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

    pub fn tip_angle(&self) -> f32 {
        self.style.tip_angle
    }

    pub fn payload(&self) -> Option<&E> {
        self.payload.as_ref()
    }

    pub fn style(&self) -> &StyleEdge {
        &self.style
    }

    pub fn color(&self, ctx: &Context) -> Color32 {
        if self.selected {
            return ctx.style().visuals.widgets.hovered.fg_stroke.color;
        }

        ctx.style()
            .visuals
            .gray_out(ctx.style().visuals.widgets.inactive.fg_stroke.color)
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

    pub fn set_selected(&mut self, selected: bool) {
        self.selected = selected;
    }

    pub fn selected(&self) -> bool {
        self.selected
    }
}
