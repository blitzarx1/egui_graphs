use egui_graphs::{Edge, Graph, Node};
use petgraph::visit::{EdgeRef, IntoEdgeReferences, IntoNodeReferences};
use serde_json;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
struct TestNodePayload {
    value: i32,
    label: String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
struct TestEdgePayload {
    weight: i32,
    kind: String,
}

#[test]
fn test_serialize_deserialize_node() {
    let payload = TestNodePayload {
        value: 42,
        label: "A".to_string(),
    };
    let node = Node::<TestNodePayload, TestEdgePayload>::new(payload.clone());
    let json = serde_json::to_string(&node).expect("serialize node");

    let node2: Node<TestNodePayload, TestEdgePayload> =
        serde_json::from_str(&json).expect("deserialize node");

    assert_eq!(node2.color(), node.color());
    assert_eq!(node2.location(), node.location());
    assert_eq!(node2.payload(), node.payload());
    assert_eq!(node2.label(), node.label());
    assert_eq!(node2.selected(), node.selected());
    assert_eq!(node2.dragged(), node.dragged());
    assert_eq!(node2.hovered(), node.hovered());
}

#[test]
fn test_serialize_deserialize_edge() {
    let payload = TestEdgePayload {
        weight: 7,
        kind: "test".to_string(),
    };
    let edge = Edge::<TestNodePayload, TestEdgePayload>::new(payload.clone());
    let json = serde_json::to_string(&edge).expect("serialize edge");
    let edge2: Edge<TestNodePayload, TestEdgePayload> =
        serde_json::from_str(&json).expect("deserialize edge");
    assert_eq!(edge2.payload(), edge.payload());
    assert_eq!(edge2.props().label, edge.props().label);
    assert_eq!(edge2.props().order, edge.props().order);
    assert_eq!(edge2.props().selected, edge.props().selected);
}

#[test]
fn test_serialize_deserialize_graph() {
    use petgraph::stable_graph::StableGraph;
    let sg: StableGraph<
        Node<TestNodePayload, TestEdgePayload>,
        Edge<TestNodePayload, TestEdgePayload>,
    > = StableGraph::default();
    let mut graph = Graph::new(sg);
    let n1 = graph.add_node(TestNodePayload {
        value: 1,
        label: "A".to_string(),
    });
    let n2 = graph.add_node(TestNodePayload {
        value: 2,
        label: "B".to_string(),
    });
    graph.add_edge(
        n1,
        n2,
        TestEdgePayload {
            weight: 42,
            kind: "test".to_string(),
        },
    );
    let json = serde_json::to_string(&graph).expect("serialize graph");
    let graph2: Graph<TestNodePayload, TestEdgePayload> =
        serde_json::from_str(&json).expect("deserialize graph");
    // Compare node and edge counts
    assert_eq!(graph2.g().node_count(), graph.g().node_count());
    assert_eq!(graph2.g().edge_count(), graph.g().edge_count());

    // Compare node payloads
    for node_ref in graph.g().node_references() {
        let (idx, node) = node_ref;
        let node2 = graph2.g().node_weight(idx).expect("node exists");
        assert_eq!(node.payload(), node2.payload());
    }

    // Compare edge payloads
    for edge_ref in graph.g().edge_references() {
        let edge = edge_ref.weight();
        let idx = edge_ref.id();
        let edge2 = graph2.g().edge_weight(idx).expect("edge exists");
        assert_eq!(edge.payload(), edge2.payload());
    }
}
