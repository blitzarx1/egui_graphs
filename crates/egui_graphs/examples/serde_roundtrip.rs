use eframe::{run_native, App, CreationContext};
use egui::{Color32, RichText};
use egui_graphs::{DefaultGraphView, Edge, Graph, Node, SettingsStyle};
use petgraph::visit::{EdgeRef, IntoEdgeReferences, IntoNodeReferences};

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
struct NodePayload {
    value: i32,
    label: String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
struct EdgePayload {
    weight: i32,
    kind: String,
}

struct Roundtrip {
    graph: Graph<NodePayload, EdgePayload>,
    json: String,
}

fn verify_node_roundtrip() {
    let node = Node::<NodePayload, EdgePayload>::new(NodePayload {
        value: 42,
        label: "A".to_owned(),
    });
    let json = serde_json::to_string(&node).expect("serialize node");
    let decoded: Node<NodePayload, EdgePayload> =
        serde_json::from_str(&json).expect("deserialize node");

    assert_eq!(decoded.color(), node.color());
    assert_eq!(decoded.location(), node.location());
    assert_eq!(decoded.payload(), node.payload());
    assert_eq!(decoded.label(), node.label());
    assert_eq!(decoded.selected(), node.selected());
    assert_eq!(decoded.dragged(), node.dragged());
    assert_eq!(decoded.hovered(), node.hovered());
}

fn verify_edge_roundtrip() {
    let edge = Edge::<NodePayload, EdgePayload>::new(EdgePayload {
        weight: 7,
        kind: "test".to_owned(),
    });
    let json = serde_json::to_string(&edge).expect("serialize edge");
    let decoded: Edge<NodePayload, EdgePayload> =
        serde_json::from_str(&json).expect("deserialize edge");

    assert_eq!(decoded.payload(), edge.payload());
    assert_eq!(decoded.props().label, edge.props().label);
    assert_eq!(decoded.props().order, edge.props().order);
    assert_eq!(decoded.props().selected, edge.props().selected);
}

fn roundtrip_graph() -> Roundtrip {
    let mut graph: Graph<NodePayload, EdgePayload> = Graph::new();
    let first = graph.add_node_with_label(
        NodePayload {
            value: 1,
            label: "A".to_owned(),
        },
        "A".to_owned(),
    );
    let second = graph.add_node_with_label(
        NodePayload {
            value: 2,
            label: "B".to_owned(),
        },
        "B".to_owned(),
    );
    graph
        .node_mut(first)
        .expect("first node exists")
        .set_color(Color32::LIGHT_BLUE);
    graph
        .node_mut(second)
        .expect("second node exists")
        .set_color(Color32::LIGHT_GREEN);
    graph.add_edge_with_label(
        first,
        second,
        EdgePayload {
            weight: 42,
            kind: "test".to_owned(),
        },
        "A to B".to_owned(),
    );

    let json = serde_json::to_string_pretty(&graph).expect("serialize graph");
    let decoded: Graph<NodePayload, EdgePayload> =
        serde_json::from_str(&json).expect("deserialize graph");

    assert_eq!(decoded.g().node_count(), graph.g().node_count());
    assert_eq!(decoded.g().edge_count(), graph.g().edge_count());

    for (index, node) in graph.g().node_references() {
        assert_eq!(
            node.payload(),
            decoded
                .g()
                .node_weight(index)
                .expect("node exists")
                .payload()
        );
    }
    for edge in graph.g().edge_references() {
        assert_eq!(
            edge.weight().payload(),
            decoded
                .g()
                .edge_weight(edge.id())
                .expect("edge exists")
                .payload()
        );
    }

    Roundtrip {
        graph: decoded,
        json,
    }
}

fn roundtrip_all() -> Roundtrip {
    verify_node_roundtrip();
    verify_edge_roundtrip();
    roundtrip_graph()
}

struct RoundtripApp {
    roundtrip: Roundtrip,
}

impl RoundtripApp {
    fn new(_: &CreationContext<'_>) -> Self {
        Self {
            roundtrip: roundtrip_all(),
        }
    }
}

impl App for RoundtripApp {
    fn ui(&mut self, ui: &mut egui::Ui, _: &mut eframe::Frame) {
        egui::Panel::top("info_panel").show(ui, render_info);

        egui::Panel::right("serialized_json")
            .default_size(360.0)
            .resizable(true)
            .show(ui, |ui| {
                ui.heading("Serialized JSON");
                ui.label(RichText::new("Roundtrip successful").color(Color32::LIGHT_GREEN));
                ui.monospace(format!("nodes: {}", self.roundtrip.graph.node_count()));
                ui.monospace(format!("edges: {}", self.roundtrip.graph.edge_count()));
                ui.separator();
                egui::ScrollArea::both().show(ui, |ui| {
                    ui.monospace(&self.roundtrip.json);
                });
            });

        egui::CentralPanel::default().show(ui, |ui| {
            DefaultGraphView::new()
                .with_styles(&SettingsStyle::default().with_labels_always(true))
                .show(ui, &mut self.roundtrip.graph);
        });
    }
}

fn render_info(ui: &mut egui::Ui) {
    ui.group(|ui| {
        ui.vertical(|ui| {
            ui.horizontal_wrapped(|ui| {
                ui.label(RichText::new("Serde roundtrip").strong());
                ui.label("serializes nodes, edges, and a graph, then restores them from JSON.");
            });
            ui.horizontal_wrapped(|ui| {
                ui.label("*");
                ui.label("The center panel renders the");
                ui.label(RichText::new("deserialized Graph").code());
                ui.label("rather than the original value.");
            });
            ui.horizontal_wrapped(|ui| {
                ui.label("*");
                ui.label("The right panel shows the exact JSON used for the roundtrip.");
            });
        });
    });
}

fn main() {
    run_native(
        "serde roundtrip",
        eframe::NativeOptions::default(),
        Box::new(|context| Ok(Box::new(RoundtripApp::new(context)))),
    )
    .unwrap();
}

#[cfg(test)]
mod tests {
    #[test]
    fn serde_roundtrips() {
        super::roundtrip_all();
    }
}
