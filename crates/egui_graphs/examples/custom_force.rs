use eframe::{run_native, App, CreationContext};
use egui::{Panel, Rect, RichText, Vec2};
use egui_graphs::{
    generate_simple_digraph, DisplayEdge, DisplayNode, Extra, ExtraForce,
    FruchtermanReingoldWithExtras, FruchtermanReingoldWithExtrasState, Graph, GraphView,
    LayoutForceDirected,
};
use petgraph::EdgeType;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default)]
struct PullToCenter;

#[derive(Clone, Debug, Deserialize, Serialize)]
struct PullToCenterParams {
    strength: f32,
}

impl Default for PullToCenterParams {
    fn default() -> Self {
        Self { strength: 0.1 }
    }
}

impl ExtraForce for PullToCenter {
    type Params = PullToCenterParams;

    fn apply<N, E, Ty, Ix, Dn, De>(
        params: &Self::Params,
        graph: &Graph<N, E, Ty, Ix, Dn, De>,
        indices: &[petgraph::stable_graph::NodeIndex<Ix>],
        displacements: &mut [Vec2],
        area: Rect,
        _k: f32,
    ) where
        N: Clone,
        E: Clone,
        Ty: EdgeType,
        Ix: petgraph::csr::IndexType,
        Dn: DisplayNode<N, E, Ty, Ix>,
        De: DisplayEdge<N, E, Ty, Ix, Dn>,
    {
        for (position, index) in indices.iter().enumerate() {
            let location = graph.g().node_weight(*index).unwrap().location();
            displacements[position] += (area.center() - location) * params.strength;
        }
    }
}

type Extras = (Extra<PullToCenter, true>, ());
type State = FruchtermanReingoldWithExtrasState<Extras>;
type Algorithm = FruchtermanReingoldWithExtras<Extras>;
type Layout = LayoutForceDirected<Algorithm>;

struct CustomForceApp {
    graph: Graph,
}

impl CustomForceApp {
    fn new(_: &CreationContext<'_>) -> Self {
        let mut graph = Graph::from(&generate_simple_digraph());
        let node_count = graph.node_count();

        for (index, node) in graph.g_mut().node_weights_mut().enumerate() {
            let angle = std::f32::consts::TAU * index as f32 / node_count as f32;
            node.set_location(egui::pos2(angle.cos() * 100.0, angle.sin() * 100.0));
        }

        Self { graph }
    }
}

impl App for CustomForceApp {
    fn ui(&mut self, ui: &mut egui::Ui, _: &mut eframe::Frame) {
        Panel::top("info_panel").show(ui, render_info);

        egui::CentralPanel::default().show(ui, |ui| {
            GraphView::<State, Layout>::new().show(ui, &mut self.graph);
        });
    }
}

fn render_info(ui: &mut egui::Ui) {
    ui.group(|ui| {
        ui.vertical(|ui| {
            ui.horizontal_wrapped(|ui| {
                ui.label(RichText::new("Custom force").strong());
                ui.label("implements");
                ui.label(RichText::new("ExtraForce").code());
                ui.label("and composes it with");
                ui.label(RichText::new("FruchtermanReingold").code());
                ui.label("using");
                ui.label(RichText::new("FruchtermanReingoldWithExtras").code());
                ui.label(".");
            });

            ui.horizontal_wrapped(|ui| {
                ui.label("*");
                ui.label(RichText::new("PullToCenter").code());
                ui.label("adds a displacement toward the center on every layout step.");
            });
        });
    });
}

fn main() {
    run_native(
        "custom force",
        eframe::NativeOptions::default(),
        Box::new(|context| Ok(Box::new(CustomForceApp::new(context)))),
    )
    .unwrap();
}
