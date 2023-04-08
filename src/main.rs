use std::time::Instant;

use eframe::{run_native, App, CreationContext};
use egui::{pos2, Color32, Context, Pos2, Response, Stroke, Ui, Vec2};
use fdg_sim::{ForceGraph, ForceGraphHelper, Simulation, SimulationParameters};
use petgraph::{stable_graph::IndexType, Graph};
use rand::seq::SliceRandom;

const CNT: usize = 100;

const NODE_RADIUS: f32 = 5.;
const EDGE_WIDTH: f32 = 2.;
const NODE_COLOR: Color32 = Color32::from_rgb(255, 255, 255);
const EDGE_COLOR: Color32 = Color32::from_rgb(128, 128, 128);

pub struct MyApp {
    simulation: Simulation<(), ()>,

    fps: usize,
    fps_accumulator: usize,
    last_fps_point: Instant,

    cursor_pos: Pos2,
    zoom: f32,
    translation: Vec2,
}

impl MyApp {
    fn new(_: &CreationContext<'_>) -> Self {
        let mut graph = Graph::<_, ()>::new();

        let mut nodes = vec![];
        (0..CNT).for_each(|_| {
            nodes.push(graph.add_node(()));
        });

        let mut rng = rand::thread_rng();
        for _ in 0..CNT {
            let mut nodes = nodes.clone();
            nodes.shuffle(&mut rng);
            let (a, b) = nodes.split_at(2);
            graph.add_edge(a[0], b[0], ());
        }
        let mut force_graph: ForceGraph<(), ()> = ForceGraph::default();
        let node_indices: Vec<_> = graph.node_indices().collect();
        for node in node_indices.iter() {
            force_graph.add_force_node(format!("{:?}", node.index()), ());
        }

        for edge in graph.edge_indices() {
            let (source, target) = graph.edge_endpoints(edge).unwrap();
            force_graph.add_edge(source, target, ());
        }

        let simulation = Simulation::from_graph(force_graph, SimulationParameters::default());

        Self {
            simulation,
            fps: 0,
            fps_accumulator: 0,
            last_fps_point: Instant::now(),
            zoom: 1.,
            cursor_pos: Pos2::ZERO,
            translation: Vec2::new(400., 300.),
        }
    }

    fn handle_interactions(&mut self, ui: &mut Ui, response: &Response) {
        ui.input(|i| {
            if let Some(pointer_pos) = i.pointer.hover_pos() {
                self.cursor_pos = pointer_pos + self.translation;
            }

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

impl App for MyApp {
    fn update(&mut self, ctx: &Context, _: &mut eframe::Frame) {
        egui::SidePanel::left("side_panel").show(ctx, |ui| {
            ui.label("* left click and drag to move the graph");
            ui.label("* ctrl + mouse wheel to zoom in and out");
            egui::TopBottomPanel::bottom("side_panel_footer").show_inside(ui, |ui| {
                ui.vertical(|ui| {
                    ui.label(format!("fps: {}", self.fps));
                    ui.label(format!("zoom: {}", self.zoom));
                    ui.label(format!("translation: {:?}", self.translation));
                    ui.label(format!("cursor: {:?}", self.cursor_pos))
                });
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            let (response, painter) =
                ui.allocate_painter(ui.available_size(), egui::Sense::click_and_drag());

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

                let zoomed_node_radius_vec =
                    Vec2::new(zoomed_node_radius, zoomed_node_radius) * dir;
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
        });

        ctx.request_repaint();

        if self.last_fps_point.elapsed().as_secs_f32() > 1.0 {
            self.fps = self.fps_accumulator;
            self.fps_accumulator = 0;
            self.last_fps_point = Instant::now();
        } else {
            self.fps_accumulator += 1;
        }
    }
}

fn main() {
    let native_options = eframe::NativeOptions::default();
    run_native(
        "egui-graph",
        native_options,
        Box::new(|cc| Box::new(MyApp::new(cc))),
    );
}
