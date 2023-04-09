use egui::{Color32, Painter, Response, Sense, Stroke, Ui, Vec2, Widget};
use fdg_sim::{ForceGraph, ForceGraphHelper, Simulation, SimulationParameters};
use petgraph::visit::IntoNodeReferences;

const NODE_RADIUS: f32 = 5.;
const EDGE_WIDTH: f32 = 2.;
const NODE_COLOR: Color32 = Color32::from_rgb(255, 255, 255);
const EDGE_COLOR: Color32 = Color32::from_rgb(128, 128, 128);
const MAX_ITERATIONS: u32 = 500;

pub struct Graph<N: Clone, E: Clone> {
    simulation: Simulation<N, E>,
    iterations: u32,

    zoom: f32,
    translation: Vec2,
    last_canvas_size: Vec2,
}

impl<N: Clone, E: Clone> Graph<N, E> {
    pub fn new(input_graph: petgraph::graph::Graph<N, E>) -> Self {
        let simulation = Simulation::from_graph(
            Graph::build_force_graph(input_graph),
            SimulationParameters::default(),
        );
        Self {
            simulation,
            iterations: 0,

            zoom: 1.,
            translation: Vec2::ZERO,
            last_canvas_size: Vec2::ZERO,
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
                let mouse_pos = match i.pointer.hover_pos() {
                    Some(mouse_pos) => mouse_pos - response.rect.min,
                    None => Vec2::ZERO,
                };
                let graph_mouse_pos = (mouse_pos - self.translation) / self.zoom;
                let new_zoom = self.zoom * zoom_delta;
                let zoom_ratio = new_zoom / self.zoom;
                self.zoom = new_zoom;
                self.translation += (1. - zoom_ratio) * graph_mouse_pos * self.zoom;
            }
        });

        if response.dragged() {
            self.translation += response.drag_delta();
        }

        self.last_canvas_size = response.rect.size();
    }

    fn update_node_position(&self, original_pos: Vec2) -> Vec2 {
        original_pos * self.zoom + self.translation
    }

    fn handle_size_change(&mut self, response: &Response) {
        if self.last_canvas_size != response.rect.size() {
            let diff = self.last_canvas_size - response.rect.size();
            self.translation -= diff / 2.;
        }
    }

    fn compute_positions(&self) -> Vec<Vec2> {
        self.simulation
            .get_graph()
            .node_weights()
            .map(|node| self.update_node_position(Vec2::new(node.location.x, node.location.y)))
            .collect::<Vec<_>>()
    }

    fn draw_nodes_and_edges(&self, p: Painter, positions: &Vec<Vec2>) {
        let edge_width = EDGE_WIDTH * self.zoom;
        let node_radius = NODE_RADIUS * self.zoom;

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

            let zoomed_node_radius_vec = Vec2::new(node_radius, node_radius) * dir;
            let tip = pos_start + vec - zoomed_node_radius_vec;

            let angle = std::f32::consts::TAU / 50.;
            let tip_length = node_radius * 3.;

            let stroke = Stroke::new(edge_width, EDGE_COLOR);
            p.line_segment([pos_start, tip], stroke);
            p.line_segment([tip, tip - tip_length * rotate_vector(dir, angle)], stroke);
            p.line_segment([tip, tip - tip_length * rotate_vector(dir, -angle)], stroke);
        });

        // Draw nodes
        for pos in positions {
            p.circle_filled(pos.to_pos2(), node_radius, NODE_COLOR);
        }
    }

    fn update_simulation(&mut self, ui: &Ui) {
        // TODO: better use some kind of graph stability measure
        // instead of a fixed number of iterations
        if self.iterations > MAX_ITERATIONS {
            return;
        }

        self.simulation.update(0.035);
        ui.ctx().request_repaint();
        self.iterations += 1;
    }
}

impl<N: Clone, E: Clone> Widget for &mut Graph<N, E> {
    fn ui(self, ui: &mut Ui) -> Response {
        let (response, painter) = ui.allocate_painter(ui.available_size(), Sense::click_and_drag());

        self.update_simulation(ui);
        self.handle_size_change(&response);
        self.draw_nodes_and_edges(painter, &self.compute_positions());
        self.handle_interactions(ui, &response);

        response
    }
}

fn rotate_vector(vec: Vec2, angle: f32) -> Vec2 {
    let cos = angle.cos();
    let sin = angle.sin();
    Vec2::new(cos * vec.x - sin * vec.y, sin * vec.x + cos * vec.y)
}
