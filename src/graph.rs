use std::collections::HashMap;

use crate::{
    elements_props::{EdgeProps, NodeProps},
    settings::Settings,
};
use egui::{Painter, Pos2, Rect, Response, Sense, Stroke, Ui, Vec2, Widget};
use fdg_sim::{glam::Vec3, ForceGraph, ForceGraphHelper, Simulation, SimulationParameters};
use petgraph::{stable_graph::NodeIndex, visit::IntoNodeReferences};

const SCREEN_PADDING: f32 = 0.3;
const MAX_ITERATIONS: u32 = 500;
const ZOOM_STEP: f32 = 0.1;
const SIMULATION_DT: f32 = 0.035;

pub struct Graph<N: Clone, E: Clone> {
    simulation: Simulation<N, E>,
    iterations: u32,

    /// current zoom factor
    zoom: f32,
    /// current pan offset
    pan: Vec2,
    /// current canvas dimensions
    canvas: Rect,

    /// index of the node that is currently being dragged
    node_dragged: Option<usize>,

    nodes_props: HashMap<usize, NodeProps>,
    edges_props: HashMap<[usize; 2], EdgeProps>,

    // TODO: combine next block into settings datastructure with default values
    // and use in the constructor
    simulation_autofit: bool,
    simulation_drag: bool,

    /// indicates if the graph was fitted to the screen on the first iteration
    first_fit: bool,

    positions: Vec<Vec2>,
    top_left_pos: Vec2,
    down_right_pos: Vec2,
}

impl<N: Clone, E: Clone> Graph<N, E> {
    pub fn new(input_graph: petgraph::graph::Graph<N, E>, settings: Settings) -> Self {
        let node_count = input_graph.node_count();
        let edge_count = input_graph.edge_count();

        let mut nodes_props = HashMap::with_capacity(node_count);
        let mut edges_props = HashMap::with_capacity(edge_count);
        let mut force_graph = ForceGraph::with_capacity(node_count, edge_count);

        input_graph.node_references().for_each(|(i, n)| {
            nodes_props.insert(i.index(), NodeProps::default());
            force_graph.add_force_node(format!("{}", i.index()).as_str(), n.clone());
        });

        input_graph.edge_indices().for_each(|e| {
            let (source, target) = input_graph.edge_endpoints(e).unwrap();
            edges_props.insert([source.index(), target.index()], EdgeProps::default());
            force_graph.add_edge(source, target, input_graph.edge_weight(e).unwrap().clone());
        });

        let simulation = Simulation::from_graph(force_graph, SimulationParameters::default());

        let mut graph = Self {
            simulation,
            iterations: Default::default(),

            zoom: 1.,
            pan: Default::default(),
            canvas: Rect::from_min_max(Pos2::default(), Pos2::default()),

            node_dragged: Default::default(),

            nodes_props,
            edges_props,

            simulation_autofit: settings.simulation_autofit,
            simulation_drag: settings.simulation_drag,

            first_fit: false,

            positions: Default::default(),
            top_left_pos: Default::default(),
            down_right_pos: Default::default(),
        };

        graph.compute_positions();

        graph
    }

    /// Enables or disables the simulation autofit. If enabled, the graph will be fitted to the screen
    /// on every simulation frame update.
    pub fn set_simulation_autofit(&mut self, simulation_autofit: bool) {
        self.simulation_autofit = simulation_autofit;
    }

    /// Enables or disables the simulation drag. If enabled, the graph will start simulation on every
    /// node drag.
    pub fn set_simulation_drag(&mut self, simulation_drag: bool) {
        self.simulation_drag = simulation_drag;
    }

    /// Pans and zooms the graph to fit the screen
    pub fn fit_screen(&mut self) {
        // calculate graph dimensions with decorative padding
        let graph_size = (self.down_right_pos - self.top_left_pos) * (1. + SCREEN_PADDING);
        let (width, height) = (graph_size.x, graph_size.y);

        // calculate canvas dimensions
        let canvas_size = self.canvas.size();
        let (canvas_width, canvas_height) = (canvas_size.x, canvas_size.y);

        // calculate zoom factors for x and y to fit the graph inside the canvas
        let zoom_x = canvas_width / width;
        let zoom_y = canvas_height / height;

        // choose the minimum of the two zoom factors to avoid distortion
        let new_zoom = zoom_x.min(zoom_y);

        // calculate the zoom delta and call handle_zoom to adjust the zoom factor
        let zoom_delta = new_zoom / self.zoom - 1.0;
        self.handle_zoom(zoom_delta, None);

        // calculate the center of the graph and the canvas
        let graph_center = (self.top_left_pos + self.down_right_pos) / 2.0;

        // adjust the pan value to align the centers of the graph and the canvas
        self.pan = self.canvas.center().to_vec2() - graph_center * self.zoom;
    }

    fn handle_all_interactions(&mut self, ui: &mut Ui, response: &Response) {
        if !self.interactions_allowed() {
            return;
        }

        ui.input(|i| {
            let delta = i.zoom_delta();
            if delta == 1. {
                return;
            }
            let step = ZOOM_STEP * (1. - delta).signum();
            self.handle_zoom(step, i.pointer.hover_pos());
        });

        self.handle_drags(response);
    }

    fn start_simulation(&mut self) {
        self.iterations = 0;
    }

    fn handle_zoom(&mut self, delta: f32, zoom_center: Option<Pos2>) {
        let center_pos = match zoom_center {
            Some(center_pos) => center_pos - self.canvas.min,
            None => Vec2::ZERO,
        };
        let graph_center_pos = (center_pos - self.pan) / self.zoom;
        let factor = 1. + delta;
        let new_zoom = self.zoom * factor;

        self.nodes_props
            .iter_mut()
            .for_each(|(_, node_props)| node_props.radius *= factor);
        self.edges_props
            .iter_mut()
            .for_each(|(_, edge_props)| edge_props.width *= factor);
        self.pan += (1. - factor) * graph_center_pos * new_zoom;
        self.zoom = new_zoom;
    }

    fn handle_drags(&mut self, response: &Response) {
        if response.drag_started() {
            let node_idx = self.positions.iter().enumerate().position(|(idx, pos)| {
                (*pos - response.hover_pos().unwrap().to_vec2()).length()
                    <= self.nodes_props.get(&idx).unwrap().radius
            });
            if let Some(node_idx) = node_idx {
                self.node_dragged = Some(node_idx);
            }
        }

        if response.dragged() {
            match self.node_dragged {
                // if we are dragging a node, we should update its position in the graph
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

                    if self.simulation_drag {
                        self.start_simulation();
                    }
                }
                // if we are not dragging a node, we should pan the graph
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

    fn compute_positions(&mut self) {
        let (mut min_x, mut min_y) = (0., 0.);
        let (mut max_x, mut max_y) = (0., 0.);

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

            let end_node_props = self.nodes_props.get(&idx_end).unwrap();
            let node_radius_vec = Vec2::new(end_node_props.radius, end_node_props.radius) * dir;
            let tip = pos_start + vec - node_radius_vec;

            let angle = std::f32::consts::TAU / 50.;
            let tip_length = end_node_props.radius * 3.;

            let edge_props = self.edges_props.get(&[idx_start, idx_end]).unwrap();
            let stroke = Stroke::new(edge_props.width, edge_props.color);
            p.line_segment([pos_start, tip], stroke);
            p.line_segment([tip, tip - tip_length * rotate_vector(dir, angle)], stroke);
            p.line_segment([tip, tip - tip_length * rotate_vector(dir, -angle)], stroke);
        });

        // Draw nodes
        self.positions.iter().enumerate().for_each(|(idx, pos)| {
            let node_props = self.nodes_props.get(&idx).unwrap();
            p.circle_filled(pos.to_pos2(), node_props.radius, node_props.color);
        });
    }

    fn simulation_finished(&self) -> bool {
        self.iterations > MAX_ITERATIONS
    }

    fn update_simulation(&mut self) -> bool {
        // TODO: better use some kind of graph stability measure
        // instead of a fixed number of iterations
        match self.simulation_finished() {
            true => false,
            false => {
                self.simulation.update(SIMULATION_DT);
                self.iterations += 1;

                true
            }
        }
    }

    fn interactions_allowed(&self) -> bool {
        !self.simulation_autofit
    }
}

impl<N: Clone, E: Clone> Widget for &mut Graph<N, E> {
    fn ui(self, ui: &mut Ui) -> Response {
        // TODO: dont store state in the widget, instead store in Ui
        let (response, painter) = ui.allocate_painter(ui.available_size(), Sense::click_and_drag());
        self.canvas = response.rect;

        if !self.first_fit {
            self.fit_screen();
            self.first_fit = true;
        }

        self.handle_all_interactions(ui, &response);
        self.compute_positions();
        self.draw_nodes_and_edges(painter);

        if self.simulation_autofit && !self.simulation_finished() {
            self.fit_screen()
        }

        if self.update_simulation() {
            ui.ctx().request_repaint();
        };

        response
    }
}

fn rotate_vector(vec: Vec2, angle: f32) -> Vec2 {
    let cos = angle.cos();
    let sin = angle.sin();
    Vec2::new(cos * vec.x - sin * vec.y, sin * vec.x + cos * vec.y)
}
