use egui::{Color32, Painter, Pos2, Response, Sense, Stroke, Ui, Vec2, Widget};
use fdg_sim::{glam::Vec3, ForceGraph, ForceGraphHelper, Simulation, SimulationParameters};
use petgraph::{stable_graph::NodeIndex, visit::IntoNodeReferences};

const NODE_RADIUS: f32 = 5.;
const EDGE_WIDTH: f32 = 2.;
const NODE_COLOR: Color32 = Color32::from_rgb(255, 255, 255);
const EDGE_COLOR: Color32 = Color32::from_rgb(128, 128, 128);
const SCREEN_PADDING: f32 = 0.5;
const MAX_ITERATIONS: u32 = 500;
const ZOOM_STEP: f32 = 0.1;

pub struct ElementsSizes {
    pub node_radius: f32,
    pub edge_width: f32,
}

pub struct Graph<N: Clone, E: Clone> {
    simulation: Simulation<N, E>,
    iterations: u32,

    /// current zoom factor
    zoom: f32,
    /// current pan offset
    pan: Vec2,
    canvas_size: Vec2,

    node_dragged: Option<usize>,

    elements_sizes: ElementsSizes,
    positions: Vec<Vec2>,
    top_left_pos: Vec2,
    down_right_pos: Vec2,
}

impl<N: Clone, E: Clone> Graph<N, E> {
    pub fn new(input_graph: petgraph::graph::Graph<N, E>) -> Self {
        let simulation = Simulation::from_graph(
            Graph::build_force_graph(input_graph),
            SimulationParameters::default(),
        );
        Self {
            simulation,
            iterations: Default::default(),

            zoom: 1.,
            pan: Default::default(),
            canvas_size: Default::default(),

            elements_sizes: ElementsSizes {
                node_radius: NODE_RADIUS,
                edge_width: EDGE_WIDTH,
            },
            node_dragged: Default::default(),
            positions: Default::default(),
            top_left_pos: Default::default(),
            down_right_pos: Default::default(),
        }
    }

    /// Pans and zooms the graph to fit the screen
    pub fn fit_screen(&mut self, response: &Response) {
        // calculate graph dimensions with decorative padding
        let graph_size = (self.down_right_pos - self.top_left_pos) * (1. + SCREEN_PADDING);
        let (width, height) = (graph_size.x, graph_size.y);

        // calculate canvas dimensions
        let canvas_size = response.rect.size();
        let (canvas_width, canvas_height) = (canvas_size.x, canvas_size.y);

        // calculate zoom factors for x and y to fit the graph inside the canvas
        let zoom_x = canvas_width / width;
        let zoom_y = canvas_height / height;

        // choose the minimum of the two zoom factors to avoid distortion
        let new_zoom = zoom_x.min(zoom_y);

        // calculate the zoom delta and call handle_zoom to adjust the zoom factor
        let zoom_delta = new_zoom / self.zoom - 1.0;
        self.handle_zoom(zoom_delta, None, response);

        // calculate the center of the graph and the canvas
        let graph_center = (self.top_left_pos + self.down_right_pos) / 2.0;
        let canvas_center = canvas_size / 2.0;

        // adjust the pan value to align the centers of the graph and the canvas
        self.pan = canvas_center - graph_center * self.zoom;
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

    fn handle_all_interactions(&mut self, ui: &mut Ui, response: &Response) {
        ui.input(|i| {
            let delta = i.zoom_delta();
            if delta == 1. {
                return;
            }
            let step = ZOOM_STEP * (1. - delta).signum();
            self.handle_zoom(step, i.pointer.hover_pos(), response);
        });

        self.handle_drags(response);
        self.handle_keys(ui, response);
        self.handle_size_change(response)
    }

    fn handle_keys(&mut self, ui: &mut Ui, response: &Response) {
        ui.input(|i| {
            if i.key_pressed(egui::Key::Space) {
                self.fit_screen(response)
            }
        });
    }

    fn handle_zoom(&mut self, delta: f32, zoom_center: Option<Pos2>, response: &Response) {
        let center_pos = match zoom_center {
            Some(center_pos) => center_pos - response.rect.min,
            None => Vec2::ZERO,
        };
        let graph_center_pos = (center_pos - self.pan) / self.zoom;
        let factor = 1. + delta;
        let new_zoom = self.zoom * factor;

        self.pan += (1. - factor) * graph_center_pos * new_zoom;
        self.elements_sizes = ElementsSizes {
            node_radius: NODE_RADIUS * new_zoom,
            edge_width: EDGE_WIDTH * new_zoom,
        };
        self.zoom = new_zoom;
    }

    fn handle_drags(&mut self, response: &Response) {
        if response.drag_started() {
            let node_idx = self.positions.iter().position(|pos| {
                (*pos - response.hover_pos().unwrap().to_vec2()).length() <= NODE_RADIUS * self.zoom
            });
            if let Some(node_idx) = node_idx {
                self.iterations = 0;
                self.node_dragged = Some(node_idx);
            }
        }

        if response.dragged() {
            match self.node_dragged {
                Some(node_dragged) => {
                    let node_pos = self.positions[node_dragged];

                    // here we should update position in the graph coordinates
                    // because on every tick we recalculate node positions assuming
                    // that they are in graph coordinates

                    // convert node position from screen to graph coordinates
                    let graph_node_pos = (node_pos - self.pan) / self.zoom;

                    // apply scaled drag translation
                    let graph_dragged_pos = graph_node_pos + response.drag_delta() / self.zoom;

                    self.simulation
                        .get_graph_mut()
                        .node_weight_mut(NodeIndex::new(node_dragged))
                        .unwrap()
                        .location = Vec3::new(graph_dragged_pos.x, graph_dragged_pos.y, 0.);
                    self.iterations = 0;
                }
                None => self.pan += response.drag_delta(),
            };
        }

        if response.drag_released() {
            self.node_dragged = Default::default();
        }
    }

    // applies current pan and zoom to the graph coordinates
    fn to_screen_coords(&self, pos_in_graph_coords: Vec2) -> Vec2 {
        pos_in_graph_coords * self.zoom + self.pan
    }

    fn handle_size_change(&mut self, response: &Response) {
        if self.canvas_size != response.rect.size() {
            self.fit_screen(response);
        }
        self.canvas_size = response.rect.size();
    }

    fn compute_positions(&mut self) {
        let (mut min_x, mut min_y) = (self.top_left_pos.x, self.top_left_pos.y);
        let (mut max_x, mut max_y) = (self.down_right_pos.x, self.down_right_pos.y);

        self.positions = self
            .simulation
            .get_graph()
            .node_weights()
            .map(|n| {
                if n.location.x < min_x {
                    min_x = n.location.x;
                };
                if n.location.y < min_y {
                    min_y = n.location.y;
                };
                if n.location.x > max_x {
                    max_x = n.location.x;
                };
                if n.location.y > max_y {
                    max_y = n.location.y;
                };

                self.to_screen_coords(Vec2::new(n.location.x, n.location.y))
            })
            .collect::<Vec<_>>();

        self.top_left_pos = Vec2::new(min_x, min_y);
        self.down_right_pos = Vec2::new(max_x, max_y);
    }

    fn draw_nodes_and_edges(&self, p: Painter) {
        let node_radius = self.elements_sizes.node_radius;
        let edge_width = self.elements_sizes.edge_width;

        // draw edges
        self.simulation.get_graph().edge_indices().for_each(|edge| {
            let (start, end) = self.simulation.get_graph().edge_endpoints(edge).unwrap();

            let idx_start = start.index();
            let idx_end = end.index();

            let pos_start = self.positions[idx_start].to_pos2();
            let pos_end = self.positions[idx_end].to_pos2();

            let vec = pos_end - pos_start;
            let l = vec.length();
            let dir = vec / l;

            let node_radius_vec = Vec2::new(node_radius, node_radius) * dir;
            let tip = pos_start + vec - node_radius_vec;

            let angle = std::f32::consts::TAU / 50.;
            let tip_length = node_radius * 3.;

            let stroke = Stroke::new(edge_width, EDGE_COLOR);
            p.line_segment([pos_start, tip], stroke);
            p.line_segment([tip, tip - tip_length * rotate_vector(dir, angle)], stroke);
            p.line_segment([tip, tip - tip_length * rotate_vector(dir, -angle)], stroke);
        });

        // Draw nodes
        self.positions.iter().for_each(|pos| {
            p.circle_filled(pos.to_pos2(), node_radius, NODE_COLOR);
        });
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

        self.compute_positions();
        self.draw_nodes_and_edges(painter);

        self.update_simulation(ui);
        self.handle_all_interactions(ui, &response);

        response
    }
}

fn rotate_vector(vec: Vec2, angle: f32) -> Vec2 {
    let cos = angle.cos();
    let sin = angle.sin();
    Vec2::new(cos * vec.x - sin * vec.y, sin * vec.x + cos * vec.y)
}
