use egui::Context;
use petgraph::{stable_graph::NodeIndex, EdgeType};

use crate::{Edge, Graph, Metadata, Node, SettingsStyle};

use super::Layers;

/// Contains all data about current widget state needed for custom drawing functions.
pub struct WidgetState<'a, N: Clone, E: Clone, Ty: EdgeType> {
    pub g: &'a Graph<N, E, Ty>,
    pub style: &'a SettingsStyle,
    pub meta: &'a Metadata,
}

/// Custom node draw function. Allows to fully customize what shape would be drawn for node.
/// The function is called for every node in the graph. Parameters:
/// - egui context, is needed for computing node props and styles;
/// - node reference, contains all node data;
/// - widget state with references to graph, style and metadata;
/// - when you create a shape, add it to the layers.
pub type FnCustomNodeDraw<N, E, Ty> =
    fn(&Context, n: &Node<N>, &WidgetState<N, E, Ty>, &mut Layers);

/// Custom edge draw function. Allows to fully customize what shape would be drawn for edge.
/// The function is called for every node pair which has edges connectiong them. Parameters:
/// - egui context, is needed for computing node props and styles;
/// - start node index and end node index;
/// - vector of edges, all edges between start and end nodes;
/// - widget state with references to graph, style and metadata;
/// - when you create a shape, add it to the layers.
pub type FnCustomEdgeDraw<N, E, Ty> =
    fn(&Context, (NodeIndex, NodeIndex), Vec<&Edge<E>>, &WidgetState<N, E, Ty>, &mut Layers);
