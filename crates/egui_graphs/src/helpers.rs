use crate::{DisplayEdge, DisplayNode, Edge, Graph, Node};
use egui::Vec2;
use petgraph::{
    graph::IndexType,
    stable_graph::{NodeIndex, StableGraph},
    visit::IntoNodeReferences,
    Directed, EdgeType, Undirected,
};
use rand::Rng;
use std::collections::HashMap;

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
/// assert_eq!(result.g().node_count(), 2);
/// assert_eq!(result.g().edge_count(), 1);
///
/// let mut indxs = result.g().node_indices();
/// let result_node1 = indxs.next().unwrap();
/// let result_node2 = indxs.next().unwrap();
/// assert_eq!(*result.g().node_weight(result_node1).unwrap().payload(), "A");
/// assert_eq!(*result.g().node_weight(result_node2).unwrap().payload(), "B");
///
/// assert_eq!(*result.g().edge_weight(result.g().edge_indices().next().unwrap()).unwrap().payload(), "edge1");
///
/// assert_eq!(*result.g().node_weight(result_node1).unwrap().label().clone(), format!("node {}", result_node1.index()));
/// assert_eq!(*result.g().node_weight(result_node2).unwrap().label().clone(), format!("node {}", result_node2.index()));
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
    let mut g = Graph::new();

    let nidx_by_input_nidx = input
        .node_references()
        .map(|(input_n_idx, input_n)| {
            (
                input_n_idx,
                g.add_node_custom(input_n.clone(), &mut *node_transform),
            )
        })
        .collect::<HashMap<NodeIndex<Ix>, NodeIndex<Ix>>>();

    input.edge_indices().for_each(|input_e_idx| {
        let (input_source_n_idx, input_target_n_idx) = input.edge_endpoints(input_e_idx).unwrap();
        let input_e = input.edge_weight(input_e_idx).unwrap();

        let input_source_n = *nidx_by_input_nidx.get(&input_source_n_idx).unwrap();
        let input_target_n = *nidx_by_input_nidx.get(&input_target_n_idx).unwrap();

        g.add_edge_custom(
            input_source_n,
            input_target_n,
            input_e.clone(),
            &mut *edge_transform,
        );
    });

    g
}

/// Calculates the size of the node in the direction of the given vector
pub fn node_size<N: Clone, E: Clone, Ty: EdgeType, Ix: IndexType, D: DisplayNode<N, E, Ty, Ix>>(
    node: &Node<N, E, Ty, Ix, D>,
    dir: Vec2,
) -> f32 {
    let connector_left = node.display().closest_boundary_point(dir);
    let connector_right = node.display().closest_boundary_point(-dir);

    ((connector_right.to_vec2() - connector_left.to_vec2()) / 2.).length()
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

/// Generates a random graph with the specified number of nodes and edges.
pub fn generate_random_graph(num_nodes: usize, num_edges: usize) -> Graph {
    let mut rng = rand::rng();
    let mut graph = StableGraph::new();

    for _ in 0..num_nodes {
        graph.add_node(());
    }

    for _ in 0..num_edges {
        let source = rng.random_range(0..num_nodes);
        let target = rng.random_range(0..num_nodes);

        graph.add_edge(NodeIndex::new(source), NodeIndex::new(target), ());
    }

    to_graph(&graph)
}

/// Simple digraph for usage in examples and tests.
pub fn generate_simple_digraph() -> StableGraph<(), (), Directed> {
    let mut g = StableGraph::new();

    let a = g.add_node(());
    let b = g.add_node(());
    let c = g.add_node(());

    g.add_edge(a, b, ());
    g.add_edge(b, c, ());
    g.add_edge(c, a, ());

    g
}

/// Simple ungraph for usage in examples and tests.
pub fn generate_simple_ungraph() -> StableGraph<(), (), Undirected> {
    let mut g = StableGraph::<_, _, Undirected>::default();

    let a = g.add_node(());
    let b = g.add_node(());
    let c = g.add_node(());

    g.add_edge(a, b, ());
    g.add_edge(b, c, ());
    g.add_edge(c, a, ());

    g
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

        assert_eq!(user_g.node_count(), input_g.g().node_count());
        assert_eq!(user_g.edge_count(), input_g.g().edge_count());
        assert_eq!(user_g.is_directed(), input_g.is_directed());

        for (user_idx, input_idx) in input_g.g().node_indices().zip(user_g.node_indices()) {
            let user_n = user_g.node_weight(user_idx).unwrap();
            let input_n = input_g.g().node_weight(input_idx).unwrap();

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

        assert_eq!(user_g.node_count(), input_g.g().node_count());
        assert_eq!(user_g.edge_count(), input_g.g().edge_count());
        assert_eq!(user_g.is_directed(), input_g.is_directed());

        for (user_idx, input_idx) in input_g.g().node_indices().zip(user_g.node_indices()) {
            let user_n = user_g.node_weight(user_idx).unwrap();
            let input_n = input_g.g().node_weight(input_idx).unwrap();

            assert_eq!(*input_n.payload(), *user_n);
            assert_eq!(*input_n.label(), format!("node {}", user_idx.index()));

            assert!(!input_n.selected());
            assert!(!input_n.dragged());
        }
    }
}
