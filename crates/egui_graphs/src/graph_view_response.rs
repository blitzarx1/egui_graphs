use egui::{Pos2, Response, Vec2};
use petgraph::stable_graph::{DefaultIx, EdgeIndex, IndexType, NodeIndex};

/// A graph-specific change produced while rendering one [`crate::GraphView`] frame.
///
/// Changes are returned in occurrence order. Variants that refer to graph entities
/// retain the graph's index type, while navigation variants are index-independent.
#[derive(Clone, Debug, PartialEq)]
pub enum GraphChange<Ix: IndexType = DefaultIx> {
    Panned {
        delta: Vec2,
        new_pan: Vec2,
    },
    Zoomed {
        delta: f32,
        new_zoom: f32,
    },
    NodeMoved {
        node: NodeIndex<Ix>,
        delta: Vec2,
        new_position: Pos2,
    },
    NodeDragStarted {
        node: NodeIndex<Ix>,
    },
    NodeDragEnded {
        node: NodeIndex<Ix>,
    },
    NodeSelected {
        node: NodeIndex<Ix>,
    },
    NodeDeselected {
        node: NodeIndex<Ix>,
    },
    NodeClicked {
        node: NodeIndex<Ix>,
    },
    NodeDoubleClicked {
        node: NodeIndex<Ix>,
    },
    NodeHoverEntered {
        node: NodeIndex<Ix>,
    },
    NodeHoverExited {
        node: NodeIndex<Ix>,
    },
    EdgeClicked {
        edge: EdgeIndex<Ix>,
    },
    EdgeSelected {
        edge: EdgeIndex<Ix>,
    },
    EdgeDeselected {
        edge: EdgeIndex<Ix>,
    },
}

/// The result of rendering a [`crate::GraphView`] for one frame.
#[derive(Debug)]
pub struct GraphViewResponse<Ix: IndexType = DefaultIx> {
    /// The standard egui response for the graph's allocated area.
    pub response: Response,
    /// Graph-specific changes produced during this `show` call, in occurrence order.
    pub changes: Vec<GraphChange<Ix>>,
}
