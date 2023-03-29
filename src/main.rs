use std::time::Instant;

use eframe::{run_native, App, CreationContext};
use egui::{pos2, Color32, Context, Stroke, Vec2};
use fdg_sim::{ForceGraph, ForceGraphHelper, Simulation, SimulationParameters};
use petgraph::{stable_graph::IndexType, Graph};
use rand::seq::SliceRandom;

pub struct MyApp {
    simulation: Simulation<(), ()>,
    fps: usize,
    fps_accumulator: usize,
    last_fps_point: Instant,
    zoom: f32,
    translation: Vec2,
}

impl MyApp {
    fn new(_: &CreationContext<'_>) -> Self {
        // Create a simple graph with petgraph
        let mut graph = Graph::<_, ()>::new();

        let mut nodes = vec![];
        (0..100).for_each(|_| {
            nodes.push(graph.add_node(()));
        });

        // Randomly connect nodes 100 nodes
        let mut rng = rand::thread_rng();
        for _ in 0..100 {
            let mut nodes = nodes.clone();
            nodes.shuffle(&mut rng);
            let (a, b) = nodes.split_at(2);
            graph.add_edge(a[0], b[0], ());
        }
        // Initialize a ForceGraph with fdg_sim
        let mut force_graph: ForceGraph<(), ()> = ForceGraph::default();
        let node_indices: Vec<_> = graph.node_indices().collect();
        for node in node_indices.iter() {
            force_graph.add_force_node(format!("{:?}", node.index()), ());
        }

        for edge in graph.edge_indices() {
            let (source, target) = graph.edge_endpoints(edge).unwrap();
            force_graph.add_edge(source, target, ());
        }

        // Create a simulation from the ForceGraph
        let simulation = Simulation::from_graph(force_graph, SimulationParameters::default());

        Self {
            simulation,
            fps: 0,
            fps_accumulator: 0,
            last_fps_point: Instant::now(),
            zoom: 1.,
            translation: Vec2::ZERO,
        }
    }
}

impl App for MyApp {
    fn update(&mut self, ctx: &Context, _: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top").show(ctx, |ui| {
            ui.label(format!("fps: {}", self.fps));
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            let available_size = ui.available_size();
            let (response, painter) =
                ui.allocate_painter(available_size, egui::Sense::click_and_drag());

            ui.input(|i| {
                if i.key_pressed(egui::Key::ArrowRight) {
                    self.translation += Vec2::new(10., 0.);
                }
                if i.key_pressed(egui::Key::ArrowLeft) {
                    self.translation += Vec2::new(-10., 0.);
                }
                if i.key_pressed(egui::Key::ArrowDown) {
                    self.translation += Vec2::new(0., -10.);
                }
                if i.key_pressed(egui::Key::ArrowUp) {
                    self.translation += Vec2::new(0., 10.);
                }

                if i.key_pressed(egui::Key::PlusEquals) {
                    self.zoom *= 1.1;
                }
                if i.key_pressed(egui::Key::Minus) {
                    self.zoom *= 0.9;
                }
                let zd = i.zoom_delta();
                if zd != 0.0 {
                    self.zoom *= zd;
                }
            });

            // Handle mouse drag events for panning
            if response.dragged() {
                self.translation += response.drag_delta();
            }

            // Update the node positions based on the force-directed algorithm
            self.simulation.update(0.05);

            // Get the node positions
            let positions = self
                .simulation
                .get_graph()
                .node_weights()
                .map(|node| node.location)
                .collect::<Vec<_>>();

            // Calculate the center of the available area
            let center = available_size / 2.0;

            // Convert positions to f32 for use with egui
            let nodes = positions
                .into_iter()
                .map(|pos| {
                    let mut pos = Vec2::new(pos.x, pos.y) * self.zoom;
                    pos += self.translation;
                    (pos.x + center.x, pos.y + center.y)
                })
                .collect::<Vec<_>>();

            let mut zoomed_edge_width = 2. * self.zoom;
            if zoomed_edge_width < 1. {
                zoomed_edge_width = 1.
            }
            let mut zoomed_node_radius = 5.0 * self.zoom;
            if zoomed_node_radius < 2. {
                zoomed_node_radius = 2.
            }

            // draw edges
            self.simulation.get_graph().edge_indices().for_each(|edge| {
                let (start, end) = self.simulation.get_graph().edge_endpoints(edge).unwrap();
                painter.line_segment(
                    [
                        pos2(nodes[start.index()].0, nodes[start.index()].1),
                        pos2(nodes[end.index()].0, nodes[end.index()].1),
                    ],
                    Stroke::new(zoomed_edge_width, Color32::from_rgb(128, 128, 128)),
                );
            });

            // Draw nodes
            for (x, y) in &nodes {
                painter.circle_filled(
                    pos2(*x, *y),
                    zoomed_node_radius,
                    Color32::from_rgb(255, 255, 255),
                );
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
