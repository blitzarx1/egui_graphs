use crate::{DisplayEdge, DisplayNode, Edge, Graph, Node};
use petgraph::{
    graph::IndexType,
    stable_graph::{EdgeIndex, NodeIndex, StableGraph},
    visit::IntoNodeReferences,
    EdgeType,
};
use std::collections::HashMap;

/// Helper function which adds user's node to the [`super::Graph`] instance.
///
/// If graph is not empty it picks any node position and adds new node in the vicinity of it.
pub fn add_node<N, E, Ty, Ix, Dn, De>(g: &mut Graph<N, E, Ty, Ix, Dn, De>, n: &N) -> NodeIndex<Ix>
where
    N: Clone,
    E: Clone,
    Ty: EdgeType,
    Ix: IndexType,
    Dn: DisplayNode<N, E, Ty, Ix>,
    De: DisplayEdge<N, E, Ty, Ix, Dn>,
{
    add_node_custom(g, n, default_node_transform)
}

/// Helper function which adds user's node to the [`super::Graph`] instance with custom node transform function.
///
/// If graph is not empty it picks any node position and adds new node in the vicinity of it.
pub fn add_node_custom<N, E, Ty, Ix, Dn, De>(
    g: &mut Graph<N, E, Ty, Ix, Dn, De>,
    n: &N,
    node_transform: impl FnOnce(&mut Node<N, E, Ty, Ix, Dn>),
) -> NodeIndex<Ix>
where
    N: Clone,
    E: Clone,
    Ty: EdgeType,
    Ix: IndexType,
    Dn: DisplayNode<N, E, Ty, Ix>,
    De: DisplayEdge<N, E, Ty, Ix, Dn>,
{
    let idx = NodeIndex::new(g.g.node_count() + 1);
    let mut n = Node::new(n.clone());

    n.set_id(idx);

    node_transform(&mut n);

    g.g.add_node(n)
}

/// Helper function which adds user's edge to the [`super::Graph`] instance.
pub fn add_edge<N, E, Ty, Ix, Dn, De>(
    g: &mut Graph<N, E, Ty, Ix, Dn, De>,
    start: NodeIndex<Ix>,
    end: NodeIndex<Ix>,
    e: &E,
) -> EdgeIndex<Ix>
where
    N: Clone,
    E: Clone,
    Ty: EdgeType,
    Ix: IndexType,
    Dn: DisplayNode<N, E, Ty, Ix>,
    De: DisplayEdge<N, E, Ty, Ix, Dn>,
{
    add_edge_custom(
        g,
        start,
        end,
        e,
        default_edge_transform::<N, E, Ty, Ix, Dn, De>,
    )
}

/// Helper function which adds user's edge to the [`super::Graph`] instance with custom edge transform function.
pub fn add_edge_custom<N, E, Ty, Ix, Dn, De>(
    g: &mut Graph<N, E, Ty, Ix, Dn, De>,
    start: NodeIndex<Ix>,
    end: NodeIndex<Ix>,
    e: &E,
    edge_transform: impl FnOnce(&mut Edge<N, E, Ty, Ix, Dn, De>),
) -> EdgeIndex<Ix>
where
    N: Clone,
    E: Clone,
    Ty: EdgeType,
    Ix: IndexType,
    Dn: DisplayNode<N, E, Ty, Ix>,
    De: DisplayEdge<N, E, Ty, Ix, Dn>,
{
    let mut edge = Edge::new(e.clone());

    edge.set_id(EdgeIndex::<Ix>::new(g.g.edge_count() + 1));
    edge.set_order(g.g.edges_connecting(start, end).count());

    edge_transform(&mut edge);

    g.g.add_edge(start, end, edge)
}

/// Helper function which transforms [`petgraph::stable_graph::StableGraph`] into the [`super::Graph`] required by the [`super::GraphView`] widget.
///
/// The function creates a new `StableGraph` where nodes and edges are represented by [`super::Node`] and [`super::Edge`] respectively.
/// New nodes and edges are created with [`default_node_transform`] and [`default_edge_transform`] functions.
/// If you want to define custom transformation procedures (e.g. to use custom label for nodes), use [`to_graph_custom`] instead.
///
/// # Example
/// ```
/// use petgraph::stable_graph::StableGraph;
/// use egui_graphs::{to_graph, DefaultNodeShape, DefaultEdgeShape, Graph};
/// use egui::Pos2;
///
/// let mut g: StableGraph<&str, &str> = StableGraph::new();
/// let node1 = g.add_node("A");
/// let node2 = g.add_node("B");
/// g.add_edge(node1, node2, "edge1");
///
/// let result: Graph<_, _, _, _, DefaultNodeShape, DefaultEdgeShape> = to_graph(&g);
///
/// assert_eq!(result.g.node_count(), 2);
/// assert_eq!(result.g.edge_count(), 1);
///
/// let mut indxs = result.g.node_indices();
/// let result_node1 = indxs.next().unwrap();
/// let result_node2 = indxs.next().unwrap();
/// assert_eq!(*result.g.node_weight(result_node1).unwrap().payload(), "A");
/// assert_eq!(*result.g.node_weight(result_node2).unwrap().payload(), "B");
///
/// assert_eq!(*result.g.edge_weight(result.g.edge_indices().next().unwrap()).unwrap().payload(), "edge1");
///
/// assert_eq!(*result.g.node_weight(result_node1).unwrap().label().clone(), format!("node {}", result_node1.index()));
/// assert_eq!(*result.g.node_weight(result_node2).unwrap().label().clone(), format!("node {}", result_node2.index()));
/// ```
pub fn to_graph<N, E, Ty, Ix, Dn, De>(g: &StableGraph<N, E, Ty, Ix>) -> Graph<N, E, Ty, Ix, Dn, De>
where
    N: Clone,
    E: Clone,
    Ty: EdgeType,
    Ix: IndexType,
    Dn: DisplayNode<N, E, Ty, Ix>,
    De: DisplayEdge<N, E, Ty, Ix, Dn>,
{
    transform(g, &mut default_node_transform, &mut default_edge_transform)
}

/// The same as [`to_graph`], but allows to define custom transformation procedures for nodes and edges.
pub fn to_graph_custom<N, E, Ty, Ix, Dn, De>(
    g: &StableGraph<N, E, Ty, Ix>,
    mut node_transform: impl FnMut(&mut Node<N, E, Ty, Ix, Dn>),
    mut edge_transform: impl FnMut(&mut Edge<N, E, Ty, Ix, Dn, De>),
) -> Graph<N, E, Ty, Ix, Dn, De>
where
    N: Clone,
    E: Clone,
    Ty: EdgeType,
    Ix: IndexType,
    Dn: DisplayNode<N, E, Ty, Ix>,
    De: DisplayEdge<N, E, Ty, Ix, Dn>,
{
    transform(g, &mut node_transform, &mut edge_transform)
}

/// Default node transform function. Keeps original data and creates a new node with a random location and
/// label equal to the index of the node in the graph.
pub fn default_node_transform<
    N: Clone,
    E: Clone,
    Ty: EdgeType,
    Ix: IndexType,
    D: DisplayNode<N, E, Ty, Ix>,
>(
    node: &mut Node<N, E, Ty, Ix, D>,
) {
    node.set_label(format!("node {}", node.id().index()));
}

/// Default edge transform function. Keeps original data and creates a new edge.
pub fn default_edge_transform<
    N: Clone,
    E: Clone,
    Ty: EdgeType,
    Ix: IndexType,
    Dn: DisplayNode<N, E, Ty, Ix>,
    D: DisplayEdge<N, E, Ty, Ix, Dn>,
>(
    edge: &mut Edge<N, E, Ty, Ix, Dn, D>,
) {
    edge.set_label(format!("edge {}", edge.id().index()));
}

fn transform<N, E, Ty, Ix, Dn, De>(
    input: &StableGraph<N, E, Ty, Ix>,
    node_transform: &mut impl FnMut(&mut Node<N, E, Ty, Ix, Dn>),
    edge_transform: &mut impl FnMut(&mut Edge<N, E, Ty, Ix, Dn, De>),
) -> Graph<N, E, Ty, Ix, Dn, De>
where
    N: Clone,
    E: Clone,
    Ty: EdgeType,
    Ix: IndexType,
    Dn: DisplayNode<N, E, Ty, Ix>,
    De: DisplayEdge<N, E, Ty, Ix, Dn>,
{
    let mut g =
        StableGraph::<Node<N, E, Ty, Ix, Dn>, Edge<N, E, Ty, Ix, Dn, De>, Ty, Ix>::default();

    let nidx_by_input_nidx = input
        .node_references()
        .map(|(input_n_idx, input_n)| {
            let mut n = Node::new(input_n.clone());

            n.set_id(input_n_idx);

            node_transform(&mut n);

            (input_n_idx, g.add_node(n))
        })
        .collect::<HashMap<NodeIndex<Ix>, NodeIndex<Ix>>>();

    input.edge_indices().for_each(|input_e_idx| {
        let (input_source_n_idx, input_target_n_idx) = input.edge_endpoints(input_e_idx).unwrap();
        let input_e = input.edge_weight(input_e_idx).unwrap();

        let input_source_n = *nidx_by_input_nidx.get(&input_source_n_idx).unwrap();
        let input_target_n = *nidx_by_input_nidx.get(&input_target_n_idx).unwrap();

        let order = g.edges_connecting(input_source_n, input_target_n).count();

        let mut edge = Edge::new(input_e.clone());

        edge.set_id(input_e_idx);
        edge.set_order(order);

        edge_transform(&mut edge);

        g.add_edge(input_source_n, input_target_n, edge);
    });

    Graph::new(g)
}

#[cfg(test)]
mod tests {
    use crate::DefaultEdgeShape;
    use crate::DefaultNodeShape;

    use super::*;
    use petgraph::Directed;
    use petgraph::Undirected;

    #[test]
    fn test_to_graph_directed() {
        let mut user_g: StableGraph<_, _, Directed> = StableGraph::new();
        let n1 = user_g.add_node("Node1");
        let n2 = user_g.add_node("Node2");
        user_g.add_edge(n1, n2, "Edge1");

        let input_g = to_graph::<_, _, _, _, DefaultNodeShape, DefaultEdgeShape>(&user_g);

        assert_eq!(user_g.node_count(), input_g.g.node_count());
        assert_eq!(user_g.edge_count(), input_g.g.edge_count());
        assert_eq!(user_g.is_directed(), input_g.is_directed());

        for (user_idx, input_idx) in input_g.g.node_indices().zip(user_g.node_indices()) {
            let user_n = user_g.node_weight(user_idx).unwrap();
            let input_n = input_g.g.node_weight(input_idx).unwrap();

            assert_eq!(*input_n.payload(), *user_n);
            assert_eq!(*input_n.label(), format!("node {}", user_idx.index()));

            assert!(!input_n.selected());
            assert!(!input_n.dragged());
        }
    }

    #[test]
    fn test_to_graph_undirected() {
        let mut user_g: StableGraph<_, _, Undirected> = StableGraph::default();
        let n1 = user_g.add_node("Node1");
        let n2 = user_g.add_node("Node2");
        user_g.add_edge(n1, n2, "Edge1");

        let input_g = to_graph::<_, _, _, _, DefaultNodeShape, DefaultEdgeShape>(&user_g);

        assert_eq!(user_g.node_count(), input_g.g.node_count());
        assert_eq!(user_g.edge_count(), input_g.g.edge_count());
        assert_eq!(user_g.is_directed(), input_g.is_directed());

        for (user_idx, input_idx) in input_g.g.node_indices().zip(user_g.node_indices()) {
            let user_n = user_g.node_weight(user_idx).unwrap();
            let input_n = input_g.g.node_weight(input_idx).unwrap();

            assert_eq!(*input_n.payload(), *user_n);
            assert_eq!(*input_n.label(), format!("node {}", user_idx.index()));

            assert!(!input_n.selected());
            assert!(!input_n.dragged());
        }
    }
}
