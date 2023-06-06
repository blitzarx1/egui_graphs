use crate::{Edge, Node};
use egui::Vec2;
use petgraph::{
    stable_graph::{NodeIndex, StableGraph},
    visit::{EdgeRef, IntoEdgeReferences},
};
use rand::Rng;
use std::collections::HashMap;

const SIDE_SIZE: f32 = 250.;

/// Helper function which transforms users `petgraph::StableGraph` isntance into the version required by the `GraphView` widget.
///
/// The users  graph, `g`, can have any data type for nodes and edges, and can be either directed
/// or undirected. The function creates a new StableGraph where the nodes and edges are encapsulated into
/// Node and Edge structs respectively. Node struct contains the original node data, a randomly generated
/// location, and default values for color, selected and dragged attributes. The Edge struct encapsulates
/// the original edge data from the users graph.
///
/// # Arguments
/// * `g` - A reference to a `petgraph::StableGraph`. The graph can have any data type for nodes and edges, and
/// can be either directed or undirected.
///
/// # Returns
/// * A new `petgrhap::StableGraph` with the same topology as the input graph, but the nodes and edges encapsulated
/// into Node and Edge structs compatible as an input to `GraphView` widget.
///
/// # Example
/// ```
/// use petgraph::stable_graph::StableGraph;
/// use egui_graphs::to_input_graph;
/// use egui::Vec2;
///
/// let mut user_graph: StableGraph<&str, &str> = StableGraph::new();
/// let node1 = user_graph.add_node("A");
/// let node2 = user_graph.add_node("B");
/// user_graph.add_edge(node1, node2, "edge1");
///
/// let input_graph = to_input_graph(&user_graph);
///
/// assert_eq!(input_graph.node_count(), 2);
/// assert_eq!(input_graph.edge_count(), 1);
///
/// let mut input_indices = input_graph.node_indices();
/// let input_node_1 = input_indices.next().unwrap();
/// let input_node_2 = input_indices.next().unwrap();
/// assert_eq!(input_graph.node_weight(input_node_1).unwrap().data, Some("A"));
/// assert_eq!(input_graph.node_weight(input_node_2).unwrap().data, Some("B"));
///
/// assert_eq!(input_graph.edge_weight(input_graph.edge_indices().next().unwrap()).unwrap().data, Some("edge1"));
///
/// let loc_1 = input_graph.node_weight(input_node_1).unwrap().location;
/// let loc_2 = input_graph.node_weight(input_node_2).unwrap().location;
/// assert!(loc_1 != Vec2::ZERO);
/// assert!(loc_2 != Vec2::ZERO);
/// ```
pub fn to_input_graph<N: Clone, E: Clone, Ty: petgraph::EdgeType>(
    g: &StableGraph<N, E, Ty>,
) -> StableGraph<Node<N>, Edge<E>, Ty> {
    let mut rng = rand::thread_rng();
    let mut input_g = StableGraph::<Node<N>, Edge<E>, Ty>::default();

    let input_by_user = g
        .node_indices()
        .map(|user_n_index| {
            let user_n = &g[user_n_index];

            let data = user_n.clone();
            let location = Vec2::new(rng.gen_range(0. ..SIDE_SIZE), rng.gen_range(0. ..SIDE_SIZE));

            let input_n = Node::new(location, data);

            let input_n_index = input_g.add_node(input_n);

            (user_n_index, input_n_index)
        })
        .collect::<HashMap<NodeIndex, NodeIndex>>();

    for user_e in g.edge_references() {
        let input_e = Edge {
            data: Some(user_e.weight().clone()),
            ..Default::default()
        };
        let input_source_n = *input_by_user.get(&user_e.source()).unwrap();
        let input_target_n = *input_by_user.get(&user_e.target()).unwrap();
        input_g.add_edge(input_source_n, input_target_n, input_e);
    }

    input_g
}

#[cfg(test)]
mod tests {
    use super::*;
    use petgraph::Directed;
    use petgraph::Undirected;

    #[test]
    fn test_to_input_graph_directed() {
        let mut user_g: StableGraph<_, _, Directed> = StableGraph::new();
        let n1 = user_g.add_node("Node1");
        let n2 = user_g.add_node("Node2");
        user_g.add_edge(n1, n2, "Edge1");

        let input_g = to_input_graph(&user_g);

        assert_eq!(user_g.node_count(), input_g.node_count());
        assert_eq!(user_g.edge_count(), input_g.edge_count());

        for (user_idx, input_idx) in input_g.node_indices().zip(user_g.node_indices()) {
            let user_n = user_g.node_weight(user_idx).unwrap();
            let input_n = input_g.node_weight(input_idx).unwrap();

            assert_eq!(input_n.data, Some(*user_n));

            assert!(input_n.location.x >= 0.0 && input_n.location.x <= SIDE_SIZE);
            assert!(input_n.location.y >= 0.0 && input_n.location.y <= SIDE_SIZE);

            assert_eq!(input_n.color, None);
            assert!(!input_n.selected);
            assert!(!input_n.dragged);
        }
    }

    #[test]
    fn test_to_input_graph_undirected() {
        let mut user_g: StableGraph<_, _, Undirected> = StableGraph::default();
        let n1 = user_g.add_node("Node1");
        let n2 = user_g.add_node("Node2");
        user_g.add_edge(n1, n2, "Edge1");

        let input_g = to_input_graph(&user_g);

        assert_eq!(user_g.node_count(), input_g.node_count());
        assert_eq!(user_g.edge_count(), input_g.edge_count());

        for (user_idx, input_idx) in input_g.node_indices().zip(user_g.node_indices()) {
            let user_n = user_g.node_weight(user_idx).unwrap();
            let input_n = input_g.node_weight(input_idx).unwrap();

            assert_eq!(input_n.data, Some(*user_n));

            assert!(input_n.location.x >= 0.0 && input_n.location.x <= SIDE_SIZE);
            assert!(input_n.location.y >= 0.0 && input_n.location.y <= SIDE_SIZE);

            assert_eq!(input_n.color, None);
            assert!(!input_n.selected);
            assert!(!input_n.dragged);
        }
    }
}
