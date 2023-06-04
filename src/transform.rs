use crate::{Edge, Node};
use egui::Vec2;
use petgraph::{
    stable_graph::{NodeIndex, StableGraph},
    visit::{EdgeRef, IntoEdgeReferences},
};
use rand::Rng;
use std::collections::HashMap;

const SIDE_SIZE: f32 = 250.;

/// Transforms a StableGraph with simple node and edge weights into a StableGraph with Node and Edge structs.
///
/// The input StableGraph, `old_graph`, can have any data type for nodes and edges, and can be either directed
/// or undirected. The function creates a new StableGraph where the nodes and edges are encapsulated into
/// Node and Edge structs respectively. Node struct contains the original node data, a randomly generated
/// location (Vec2), and default values for color, selected and dragged attributes. The Edge struct encapsulates
/// the original edge data from the old graph.
///
/// # Arguments
/// * `old_graph` - A reference to a StableGraph. The graph can have any data type for nodes and edges, and
/// can be either directed or undirected.
///
/// # Returns
/// * A new StableGraph with the same topology as the input graph, but the nodes and edges encapsulated
/// into Node and Edge structs.
///
/// # Example
/// ```
/// use petgraph::stable_graph::StableGraph;
/// use egui_graphs::transform_weighted_stable_graph;
///
/// let mut graph: StableGraph<&str, &str> = StableGraph::new();
/// let node1 = graph.add_node("A");
/// let node2 = graph.add_node("B");
/// graph.add_edge(node1, node2, "edge1");
///
/// let new_graph = transform_weighted_stable_graph(&graph);
/// ```
pub fn transform_weighted_stable_graph<N: Clone, E: Clone, Ty: petgraph::EdgeType>(
    old_graph: &StableGraph<N, E, Ty>,
) -> StableGraph<Node<N>, Edge<E>, Ty> {
    let mut new_graph = StableGraph::<Node<N>, Edge<E>, Ty>::default();
    let mut rng = rand::thread_rng();

    let node_mapping: HashMap<NodeIndex, NodeIndex> = old_graph
        .node_indices()
        .map(|old_node_index| {
            let old_node = &old_graph[old_node_index];
            let new_node = Node {
                data: Some(old_node.clone()),
                location: Vec2::new(rng.gen_range(0.0..SIDE_SIZE), rng.gen_range(0.0..SIDE_SIZE)),
                ..Default::default()
            };
            let new_node_index = new_graph.add_node(new_node);
            (old_node_index, new_node_index)
        })
        .collect();

    for edge in old_graph.edge_references() {
        let new_edge = Edge {
            data: Some(edge.weight().clone()),
            ..Default::default()
        };
        let source_node = *node_mapping.get(&edge.source()).unwrap();
        let target_node = *node_mapping.get(&edge.target()).unwrap();
        new_graph.add_edge(source_node, target_node, new_edge);
    }

    new_graph
}

#[cfg(test)]
mod tests {
    use super::*;
    use petgraph::Directed;
    use petgraph::Undirected;

    #[test]
    fn test_transform_weighted_stable_directed_graph() {
        let mut old_graph: StableGraph<_, _, Directed> = StableGraph::new();
        let n1 = old_graph.add_node("Node1");
        let n2 = old_graph.add_node("Node2");
        old_graph.add_edge(n1, n2, "Edge1");

        let new_graph = transform_weighted_stable_graph(&old_graph);

        assert_eq!(old_graph.node_count(), new_graph.node_count());
        assert_eq!(old_graph.edge_count(), new_graph.edge_count());

        for (old_node_index, new_node_index) in
            new_graph.node_indices().zip(old_graph.node_indices())
        {
            let old_node = old_graph.node_weight(old_node_index).unwrap();
            let new_node = new_graph.node_weight(new_node_index).unwrap();

            assert_eq!(new_node.data, Some(old_node.clone()));

            assert!(new_node.location.x >= 0.0 && new_node.location.x <= SIDE_SIZE);
            assert!(new_node.location.y >= 0.0 && new_node.location.y <= SIDE_SIZE);

            assert_eq!(new_node.color, None);
            assert_eq!(new_node.selected, false);
            assert_eq!(new_node.dragged, false);
        }
    }

    #[test]
    fn test_transform_weighted_stable_undirected_graph() {
        let mut old_graph: StableGraph<_, _, Undirected> = StableGraph::default();
        let n1 = old_graph.add_node("Node1");
        let n2 = old_graph.add_node("Node2");
        old_graph.add_edge(n1, n2, "Edge1");

        let new_graph = transform_weighted_stable_graph(&old_graph);

        assert_eq!(old_graph.node_count(), new_graph.node_count());
        assert_eq!(old_graph.edge_count(), new_graph.edge_count());

        for (old_node_index, new_node_index) in
            new_graph.node_indices().zip(old_graph.node_indices())
        {
            let old_node = old_graph.node_weight(old_node_index).unwrap();
            let new_node = new_graph.node_weight(new_node_index).unwrap();

            assert_eq!(new_node.data, Some(old_node.clone()));

            assert!(new_node.location.x >= 0.0 && new_node.location.x <= SIDE_SIZE);
            assert!(new_node.location.y >= 0.0 && new_node.location.y <= SIDE_SIZE);

            assert_eq!(new_node.color, None);
            assert_eq!(new_node.selected, false);
            assert_eq!(new_node.dragged, false);
        }
    }
}
