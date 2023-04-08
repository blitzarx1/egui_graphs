use egui::{Color32, Response, Sense, Stroke, Ui, Vec2, Widget};
use fdg_sim::{ForceGraph, ForceGraphHelper, Simulation, SimulationParameters};
use petgraph::visit::IntoNodeReferences;

const NODE_RADIUS: f32 = 5.;
const EDGE_WIDTH: f32 = 2.;
const NODE_COLOR: Color32 = Color32::from_rgb(255, 255, 255);
const EDGE_COLOR: Color32 = Color32::from_rgb(128, 128, 128);

pub struct Graph<N: Clone, E: Clone> {
    simulation: Simulation<N, E>,

    zoom: f32,
    translation: Vec2,
}

impl<N: Clone, E: Clone> Graph<N, E> {
    pub fn new(input_graph: petgraph::graph::Graph<N, E>) -> Self {
        let simulation = Simulation::from_graph(
            Graph::build_force_graph(input_graph),
            SimulationParameters::default(),
        );
        Self {
            simulation,

            zoom: 1.,
            translation: Vec2::ZERO,
        }
    }

    fn build_force_graph(input_graph: petgraph::graph::Graph<N, E>) -> ForceGraph<N, E> {
        let mut force_graph =
            ForceGraph::<N, E>::with_capacity(input_graph.node_count(), input_graph.edge_count());

        input_graph.node_references().for_each(|(i, n)| {
            force_graph.add_force_node(format!("{}", i.index()).as_str(), n.clone());
        });

        input_graph.edge_indices().for_each(|e| {
            let (source, target) = input_graph.edge_endpoints(e).unwrap();
            force_graph.add_edge(source, target, input_graph.edge_weight(e).unwrap().clone());
        });

        force_graph
    }

    fn handle_interactions(&mut self, ui: &mut Ui, response: &Response) {
        ui.input(|i| {
            let zoom_delta = i.zoom_delta();
            if zoom_delta != 1. {
                self.zoom *= zoom_delta;
            }
        });

        if response.dragged() {
            self.translation += response.drag_delta();
        }
    }

    fn update_node_position(&self, original_pos: Vec2) -> Vec2 {
        original_pos * self.zoom + self.translation
    }
}

impl<N: Clone, E: Clone> Widget for &mut Graph<N, E> {
    fn ui(self, ui: &mut Ui) -> Response {
        let (response, painter) = ui.allocate_painter(ui.available_size(), Sense::click_and_drag());

        self.handle_interactions(ui, &response);

        // Update the node positions based on the force-directed algorithm
        self.simulation.update(0.035);

        let positions = &self
            .simulation
            .get_graph()
            .node_weights()
            .map(|node| self.update_node_position(Vec2::new(node.location.x, node.location.y)))
            .collect::<Vec<_>>();

        let zoomed_edge_width = EDGE_WIDTH * self.zoom;
        let zoomed_node_radius = NODE_RADIUS * self.zoom;

        // draw edges
        self.simulation.get_graph().edge_indices().for_each(|edge| {
            let (start, end) = self.simulation.get_graph().edge_endpoints(edge).unwrap();

            let idx_start = start.index();
            let idx_end = end.index();

            let pos_start = positions[idx_start].to_pos2();
            let pos_end = positions[idx_end].to_pos2();

            let vec = pos_end - pos_start;
            let l = vec.length();
            let dir = vec / l;

            let zoomed_node_radius_vec = Vec2::new(zoomed_node_radius, zoomed_node_radius) * dir;
            let tip = pos_start + vec - zoomed_node_radius_vec;

            let rot = eframe::emath::Rot2::from_angle(std::f32::consts::TAU / 50.);
            let tip_length = zoomed_node_radius * 3.;

            let stroke = Stroke::new(zoomed_edge_width, EDGE_COLOR);
            painter.line_segment([pos_start, tip], stroke);
            painter.line_segment([tip, tip - tip_length * (rot * dir)], stroke);
            painter.line_segment([tip, tip - tip_length * (rot.inverse() * dir)], stroke);
        });

        // Draw nodes
        for pos in positions {
            painter.circle_filled(pos.to_pos2(), zoomed_node_radius, NODE_COLOR);
        }

        ui.ctx().request_repaint();

        response
    }
}
